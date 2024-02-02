Class {
	#name : #StackToRegisterMappingCogitWithTelemetry,
	#superclass : #StackToRegisterMappingCogit,
	#instVars : [
		'telemetryLinkedSendTrampoline',
		'telemetryBlockActivationTrampoline',
		'telemetryBlockReturnTrampoline',
		'telemetryArrowReturnTrampoline',
		'telemetryQuickPrimitiveReturnTrampoline',
		'telemetryEnilopmartReturnTrampoline',
		'telemetryPrimReturnTrampoline',
		'telemetryInterpreterPrimitiveReturnTrampoline',
		'telemetryMachineCodePrimitiveReturnTrampoline'
	],
	#category : #'GToolkit-VMMaker-AddOns-Telemetry'
}

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry class >> declareCVarsIn: aCCodeGenerator [
	aCCodeGenerator
		addHeaderFileFirst: '"telemetry.h"'
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> compileEntry [
	super compileEntry.
	
	backEnd saveAndRestoreLinkRegAround: [
		self CallRT: telemetryLinkedSendTrampoline ]
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> compileFullBlockEntry [
	super compileFullBlockEntry.
	
	backEnd saveAndRestoreLinkRegAround: [
		self CallRT: telemetryBlockActivationTrampoline ]
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> compileInterpreterPrimitive: primitiveRoutine flags: flags [
	"Compile a call to an interpreter primitive.  Call the C routine with the
	 usual stack-switching dance, test the primFailCode and then either
	 return on success or continue to the method body."
	<var: #primitiveRoutine declareC: #'void (*primitiveRoutine)(void)'>
	<var: #cogPrimitiveCallState type: #'CogPrimitiveCallState *'>

	| cogPrimitiveCallState |

	(flags anyMask: PrimCallMayCallBack)
		ifTrue: [ ^ self compileInterpreterPrimitiveMayCallBack: primitiveRoutine flags: flags ].

	self 
		cCode: [ cogPrimitiveCallState := self 
						cCoerce: (self alloca: (self sizeof: CogPrimitiveCallState)) 
						to: #'CogPrimitiveCallState*' . ] 
		inSmalltalk: [cogPrimitiveCallState := CogPrimitiveCallState new].	

	"Save processor fp, sp and return pc in the interpreter's frame stack and instruction pointers"
	self genExternalizePointersForPrimitiveCall.
	"Switch to the C stack."
	self genLoadCStackPointersForPrimCall.

	(flags anyMask: PrimCallCollectsProfileSamples) ifTrue:
		["Test nextProfileTick for being non-zero and call checkProfileTick if so"
		objectMemory wordSize = 4
			ifTrue:
				[self MoveAw: coInterpreter nextProfileTickAddress R: TempReg.
				 self MoveAw: coInterpreter nextProfileTickAddress + objectMemory wordSize R: ClassReg.
				 self OrR: TempReg R: ClassReg]
			ifFalse:
				[self MoveAw: coInterpreter nextProfileTickAddress R: TempReg.
				 self CmpCq: 0 R: TempReg].
		"If set, jump to record sample call."
		cogPrimitiveCallState jmpSampleNonPrim: (self JumpNonZero: 0).
		cogPrimitiveCallState continuePostSampleNonPrim: self Label].

	"Old full prim trace is in VMMaker-eem.550 and prior"
	self recordPrimTrace ifTrue:
		[self genFastPrimTraceUsing: ClassReg and: SendNumArgsReg].

	"Clear the primFailCode and set argumentCount"
	self MoveCq: 0 R: TempReg.
	self MoveR: TempReg Aw: coInterpreter primFailCodeAddress.
	methodOrBlockNumArgs ~= 0 ifTrue:
		[self MoveCq: methodOrBlockNumArgs R: TempReg].
	self MoveR: TempReg Aw: coInterpreter argumentCountAddress.

	"If required, set primitiveFunctionPointer and newMethod"
	(flags anyMask: PrimCallNeedsPrimitiveFunction) ifTrue:
		[self MoveCw: primitiveRoutine asInteger R: TempReg.
		 primSetFunctionLabel :=
		 self MoveR: TempReg Aw: coInterpreter primitiveFunctionPointerAddress].
	(flags anyMask: PrimCallNeedsNewMethod+PrimCallMayCallBack) ifTrue:
		["The ceActivateFailingPrimitiveMethod: machinery can't handle framelessness."
		 (flags anyMask: PrimCallMayCallBack) ifTrue:
			[needsFrame := true].
		 methodLabel addDependent:
			(self annotateAbsolutePCRef:
				(self MoveCw: methodLabel asInteger R: ClassReg)).
		 self MoveMw: (self offset: CogMethod of: #methodObject) r: ClassReg R: TempReg.
		 self MoveR: TempReg Aw: coInterpreter newMethodAddress].

	"Invoke the primitive"
	self PrefetchAw: coInterpreter primFailCodeAddress.

	"Call the C primitive routine."
	backEnd genMarshallNArgs: 0 arg: 0 arg: 0 arg: 0 arg: 0.
	primInvokeInstruction := self CallFullRT: primitiveRoutine asInteger.
	backEnd genRemoveNArgsFromStack: 0.
	(flags anyMask: PrimCallCollectsProfileSamples) ifTrue:
		[self assert: (flags anyMask: PrimCallNeedsNewMethod).
		"Test nextProfileTick for being non-zero and call checkProfileTick if so"
		objectMemory wordSize = 4
			ifTrue:
				[self MoveAw: coInterpreter nextProfileTickAddress R: TempReg.
				 self MoveAw: coInterpreter nextProfileTickAddress + objectMemory wordSize R: ClassReg.
				 self OrR: TempReg R: ClassReg]
			ifFalse:
				[self MoveAw: coInterpreter nextProfileTickAddress R: TempReg.
				 self CmpCq: 0 R: TempReg].
		"If set, jump to record sample call."
		cogPrimitiveCallState jmpSamplePrim: (self JumpNonZero: 0).
		cogPrimitiveCallState continuePostSamplePrim: self Label].
	objectRepresentation maybeCompileRetryOnPrimitiveFail: primitiveIndex.
	self maybeCompileAllocFillerCheck.
	"Switch back to the Smalltalk stack.  Stack better be in either of these two states:
		success:	stackPointer ->	result (was receiver)
									arg1
									...
									argN
									return pc
		failure:						receiver
									arg1
									...
					stackPointer ->	argN
									return pc
	In either case we can push the instructionPointer or load it into the LinkRegister to reestablish the return pc"
	self MoveAw: coInterpreter instructionPointerAddress
		R: (backEnd hasLinkRegister ifTrue: [LinkReg] ifFalse: [ClassReg]).
	backEnd genLoadStackPointers.
	"Test primitive failure"
	self MoveAw: coInterpreter primFailCodeAddress R: TempReg.
	backEnd hasLinkRegister ifFalse: [self PushR: ClassReg]. "Restore return pc on CISCs"
	self flag: 'ask concrete code gen if move sets condition codes?'.
	self CmpCq: 0 R: TempReg.
	cogPrimitiveCallState jmpToFallbackCode: (self JumpNonZero: 0).
	"Fetch result from stack"
	self MoveMw: (backEnd hasLinkRegister ifTrue: [0] ifFalse: [objectMemory wordSize])
		r: SPReg
		R: ReceiverResultReg.
	
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryInterpreterPrimitiveReturnTrampoline ].
	
	self RetN: objectMemory wordSize.	"return to caller, popping receiver"

	(flags anyMask: PrimCallCollectsProfileSamples) ifTrue:
		["The sample is collected by cePrimReturnEnterCogCode for external calls"
		
		"Call ceCheckProfileTick: to record sample and then continue."
		self 
			generateCheckProfileTickFromJump: cogPrimitiveCallState jmpSamplePrim 
			returningTo: cogPrimitiveCallState continuePostSamplePrim 
			beforeCallDo: [].
		
		"Call ceCheckProfileTick: to record sample and after the primitive and then continue."
		self 
			generateCheckProfileTickFromJump: cogPrimitiveCallState jmpSampleNonPrim 
			returningTo: cogPrimitiveCallState continuePostSampleNonPrim 
			beforeCallDo: [
				self MoveCq: 0 R: TempReg. 
				self MoveR: TempReg Aw: coInterpreter newMethodAddress]].

	"Jump to restore of receiver reg and proceed to frame build for failure."
	cogPrimitiveCallState jmpToFallbackCode jmpTarget: self Label.
	"Restore receiver reg from stack.  If on RISCs ret pc is in LinkReg, if on CISCs ret pc is on stack."
	self MoveMw: objectMemory wordSize * (methodOrBlockNumArgs + (backEnd hasLinkRegister ifTrue: [0] ifFalse: [1]))
		r: SPReg
		R: ReceiverResultReg.
				
	^0
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> compileMachineCodeInterpreterPrimitive: primitiveRoutine [
	"Compile a call to a machine-code convention interpreter primitive.  Call the C routine
	 on the Smalltalk stack, assuming it consumes little or no stack space."
	<var: #primitiveRoutine declareC: 'void (*primitiveRoutine)(void)'>
	| jmpFail liveRegsMask |
	

	"for now handle functions with less than 4 arguments; our C call marshalling machinery
	 extends up to 4 arguments only, and the first argument of an mcprim is the receiver."
	self assert: methodOrBlockNumArgs <= 3.
	liveRegsMask := (methodOrBlockNumArgs > self numRegArgs
					   or: [methodOrBlockNumArgs = 0])
						ifTrue:
							[self registerMaskFor: ReceiverResultReg and: ClassReg]
						ifFalse:
							[(self numRegArgs > 1 and: [methodOrBlockNumArgs > 1])
								ifFalse: [self registerMaskFor: ReceiverResultReg and: Arg0Reg and: ClassReg]
								ifTrue: [self registerMaskFor: ReceiverResultReg and: Arg0Reg and: Arg1Reg and: ClassReg]].

	"As this is using a call, I need to store the Link register."
	backEnd hasLinkRegister 
		ifTrue: [ self MoveR: LinkReg R: Extra0Reg ].
	
	backEnd genSaveRegs: (liveRegsMask bitAnd: CallerSavedRegisterMask).
	methodOrBlockNumArgs > self numRegArgs ifTrue:
		["Wrangle args into Arg0Reg, Arg1Reg, SendNumArgsReg & ClassReg"
		 "offset := self bitCountOf: (liveRegsMask bitAnd: CallerSavedRegisterMask)."
		 self shouldBeImplemented].

	backEnd prepareStackToCallCFunctionInSmalltalkStack: methodOrBlockNumArgs + 1.
	
	backEnd
		genMarshallNArgs: methodOrBlockNumArgs + 1
		arg: ReceiverResultReg
		arg: Arg0Reg
		arg: Arg1Reg
		arg: SendNumArgsReg
		"arg: ClassReg (when we extend C call marchalling to support 5 args for replaceFrom:to:with:startingAt:".

	self CallFullRT: primitiveRoutine asInteger.

	backEnd
		genRemoveNArgsFromStack: methodOrBlockNumArgs + 1;
		returnFromCallCFunctionInSmalltalkStack: methodOrBlockNumArgs + 1;
		genRestoreRegs: (liveRegsMask bitAnd: CallerSavedRegisterMask).

	backEnd hasLinkRegister 
		ifTrue: [ self MoveR: Extra0Reg R: LinkReg ].

	self CmpCq: 0 R: backEnd cResultRegister.
	jmpFail := self JumpZero: 0.
	backEnd cResultRegister ~= ReceiverResultReg ifTrue:
		[self MoveR: backEnd cResultRegister R: ReceiverResultReg].
	
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryMachineCodePrimitiveReturnTrampoline ].
	
	self RetN: (methodOrBlockNumArgs > self numRegArgs
				ifTrue: [methodOrBlockNumArgs + 1 * objectMemory wordSize]
				ifFalse: [0]).
	jmpFail jmpTarget: self Label.
	^0
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> compileMethodBody [
	"quick primitives"
	endPC < initialPC ifTrue: [
		backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryQuickPrimitiveReturnTrampoline ].
		^0].
	
	^ super compileMethodBody
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genBlockReturn [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryBlockReturnTrampoline ].
	
	^ super genBlockReturn
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genEnilopmartReturn: forCall [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryEnilopmartReturnTrampoline ].
	
	^ super genEnilopmartReturn: forCall
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimReturn [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimReturnTrampoline ].
	
	^ super genPrimReturn
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genUpArrowReturn [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryArrowReturnTrampoline ].

	^ super genUpArrowReturn
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> generateTracingTrampolines [
	super generateTracingTrampolines.
	
	telemetryLinkedSendTrampoline :=
		self genTrampolineFor: #telemetryEmitLinkedSend:
			called: 'telemetryLinkedSendTrampoline'
			arg: ReceiverResultReg
			regsToSave: CallerSavedRegisterMask.

	telemetryArrowReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitArrowReturn
			called: 'telemetryArrowReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryBlockReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitBlockReturn
			called: 'telemetryBlockReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryQuickPrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitQuickPrimitiveReturn
			called: 'telemetryQuickPrimitiveReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryEnilopmartReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitEnilopmartReturn
			called: 'telemetryEnilopmartReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryPrimReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolinePrimReturn
			called: 'telemetryPrimReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.
			
	telemetryInterpreterPrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineInterpreterPrimitiveReturn
			called: 'telemetryInterpreterPrimitiveReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.
			
	telemetryMachineCodePrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineMachineCodePrimitiveReturn
			called: 'telemetryMachineCodePrimitiveReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryBlockActivationTrampoline :=
		self genTrampolineFor: #telemetryEmitBlockActivation
			called: 'telemetryBlockActivationTrampoline'
			regsToSave: CallerSavedRegisterMask
]
