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
		'telemetryMachineCodePrimitiveReturnTrampoline',
		'telemetryPrimitiveActivationTrampoline',
		'telemetryPrimitiveMayCallMethodsTrampoline',
		'telemetryPrimitiveDeactivationTrampoline'
	],
	#category : #'GToolkit-VMMaker-AddOns-Telemetry'
}

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry class >> declareCVarsIn: aCCodeGenerator [
	aCCodeGenerator
		addHeaderFileFirst: '"telemetry.h"'
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry class >> initializePrimitiveTable [
	| toOverride |
	super initializePrimitiveTable.
	
	toOverride := #(
		(198 genPrimitiveMarkUnwindMethodWithTelemetry)
		(199 genPrimitiveMarkHandlerMethodWithTelemetry)

		(207 genPrimitiveFullClosureValueWithTelemetry) "value[:value:value:value:] et al"
		(209 genPrimitiveFullClosureValueWithTelemetry) "valueNoContextSwitch[:value:] et al"
	).
	
	toOverride do: [ :each | primitiveTable at: each first put: nil ].
	
	self table: primitiveTable from: toOverride
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry class >> numTrampolines [
	^ super numTrampolines + (self instVarNames select: [ :each | each endsWith: #'Trampoline' ]) size
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> compileEntireMethod [
	"Compile the abstract instructions for the entire method, including blocks."
	| result activationTrampoline|
	
	regArgsHaveBeenPushed := false.
	
	self preenMethodLabel.
	self compileAbort.
	self compileEntry.
	
	activationTrampoline := 0.
	primitiveIndex = 0
		ifFalse: [
			self genTelemetryTrampolineCall: telemetryPrimitiveActivationTrampoline.
			activationTrampoline := 1 ].
	
	(result := self compilePrimitive) < 0
		ifTrue: [
			self genTelemetryTrampolineCall: telemetryPrimitiveDeactivationTrampoline.
			^ result ].
			
	activationTrampoline = 1
		ifTrue: [ self genTelemetryTrampolineCall: telemetryPrimitiveDeactivationTrampoline ].

	self compileFrameBuild.
	^ self compileMethodBody
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> compileEntry [
	super compileEntry.

	self genTelemetryTrampolineCall: telemetryLinkedSendTrampoline
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> compileFullBlockEntry [
	super compileFullBlockEntry.
	
	self genTelemetryTrampolineCall: telemetryBlockActivationTrampoline
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> compileMethodBody [
	"quick primitives"
	endPC < initialPC ifTrue: [
		self genTelemetryTrampolineCall: telemetryQuickPrimitiveReturnTrampoline.
		^0].
	
	^ super compileMethodBody
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genBlockReturn [
	self genTelemetryTrampolineCall: telemetryBlockReturnTrampoline.
	
	^ super genBlockReturn
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genEnilopmartReturn: forCall [
	self genTelemetryTrampolineCall: telemetryEnilopmartReturnTrampoline.
	
	^ super genEnilopmartReturn: forCall
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimReturn [
	<inline: true>
	self genTelemetryTrampolineCall: telemetryPrimReturnTrampoline.
	^ super genPrimReturn
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimitiveFullClosureValueWithTelemetry [
	| code |
	<option: #SistaV1BytecodeSet>
	self genTelemetryTrampolineCall: telemetryPrimitiveMayCallMethodsTrampoline.
	code := self genPrimitiveFullClosureValue.
	^ code
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimitiveMarkHandlerMethodWithTelemetry [
	self genTelemetryTrampolineCall: telemetryPrimitiveMayCallMethodsTrampoline.
	^ self genFastPrimFail
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimitiveMarkUnwindMethodWithTelemetry [
	self genTelemetryTrampolineCall: telemetryPrimitiveMayCallMethodsTrampoline.
	^ self genFastPrimFail
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimitivePerform [
	self genTelemetryTrampolineCall: telemetryPrimitiveMayCallMethodsTrampoline.
	^ super genPrimitivePerform
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genTelemetryTrampolineCall: aTrampoline [
	<var: #jumpNoTelemetry type: #'AbstractInstruction *'>
	| jumpNoTelemetry |
	
	self MoveAw: coInterpreter telemetryEnabledAddress R: TempReg.
	self CmpCq: 0 R: TempReg.

	"will generate a short jump"
	jumpNoTelemetry := self JumpZero: 0.

	backEnd saveAndRestoreLinkRegAround: [ self CallRT: aTrampoline ].
	
	jumpNoTelemetry jmpTarget: self Label
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genUpArrowReturn [
	self genTelemetryTrampolineCall: telemetryArrowReturnTrampoline.

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
			regsToSave: CallerSavedRegisterMask.

	telemetryBlockReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitBlockReturn
			called: 'telemetryBlockReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.

	telemetryQuickPrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitQuickPrimitiveReturn
			called: 'telemetryQuickPrimitiveReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.

	telemetryEnilopmartReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitEnilopmartReturn
			called: 'telemetryEnilopmartReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.

	telemetryPrimReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolinePrimReturn
			called: 'telemetryPrimReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.

	telemetryInterpreterPrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineInterpreterPrimitiveReturn
			called: 'telemetryInterpreterPrimitiveReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.
			
	telemetryMachineCodePrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineMachineCodePrimitiveReturn
			called: 'telemetryMachineCodePrimitiveReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.
			
	telemetryPrimitiveActivationTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitPrimitiveActivation
			called: 'telemetryPrimitiveActivationTrampoline'
			regsToSave: CallerSavedRegisterMask.

	telemetryPrimitiveDeactivationTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitPrimitiveDeactivation
			called: 'telemetryPrimitiveDeactivationTrampoline'
			regsToSave: CallerSavedRegisterMask.
	
	telemetryPrimitiveMayCallMethodsTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitPrimitiveMayCallMethods
			called: 'telemetryPrimitiveMayCallMethodsTrampoline'
			regsToSave: CallerSavedRegisterMask.


	telemetryBlockActivationTrampoline :=
		self genTrampolineFor: #telemetryEmitBlockActivation
			called: 'telemetryBlockActivationTrampoline'
			regsToSave: CallerSavedRegisterMask
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> traceLinkedSendOffset [
	^ super traceLinkedSendOffset
		+ backEnd jumpLongByteSize
		+ backEnd loadLiteralByteSize
		+ backEnd compareQuickConstantByteSize
]
