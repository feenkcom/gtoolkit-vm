"
Generating VM sources to test the implementation:
```
CoInterpreterWithTelemetry generate
```
"
Class {
	#name : #CoInterpreterWithTelemetry,
	#superclass : #CoInterpreterPrimitives,
	#instVars : [
		'telemetry',
		'telemetryEnabled',
		'telemetryReceiverClass',
		'telemetrySendSelector'
	],
	#classVars : [
		'ArrowReturnTrampolineSignal',
		'BaseFrameReturnSignal',
		'BlockReturnTrampolineSignal',
		'CommonCallerReturnSignal',
		'EnilopmartReturnTrampolineSignal',
		'ExternalAboutToReturnThroughSignal',
		'InterpreterCode',
		'InterpreterPrimitiveReturnTrampolineSignal',
		'InterpreterToMachineCode',
		'MachineCode',
		'MachineCodePrimitiveReturnTrampolineSignal',
		'MachineToInterpreterCode',
		'MaybeReturnToMachineCodeFrameSignal',
		'NonLocalReturnSignal',
		'PrimReturnTrampolineSignal',
		'QuickPrimitiveReturnTrampolineSignal',
		'ReturnToExecutivePostContextWithSwitchSignal',
		'ReturnToExecutivePostContextWithoutSwitchSignal',
		'ReturnToExecutiveSignal',
		'ReturnToInterpreterSignal',
		'ReturnToMachineCodeFrameSignal',
		'SlowPrimitiveReturnSignal'
	],
	#category : #'GToolkit-VMMaker-AddOns-Telemetry'
}

{ #category : #accessing }
CoInterpreterWithTelemetry class >> additionalHeadersDo: aBinaryBlock [
	"Evaluate aBinaryBlock with the names and contents of
	 any additional header files that need to be generated."
	 super additionalHeadersDo: aBinaryBlock.
	 
	aBinaryBlock
		value: 'telemetry.h'
		value: CoInterpreterTelemetry cogMethodHeader
]

{ #category : #accessing }
CoInterpreterWithTelemetry class >> ancilliaryClasses [
	^ super ancilliaryClasses, { CoInterpreterTelemetry }
]

{ #category : #accessing }
CoInterpreterWithTelemetry class >> declareCVarsIn: aCCodeGenerator [
	"Override to avoid repeating StackInterpreter's declarations and add our own extensions"
	self class == thisContext methodClass
		ifFalse: [^self]. "Don't duplicate decls in subclasses"

	aCCodeGenerator
		addHeaderFileFirst: '"telemetry.h"'.

	aCCodeGenerator
		var: #telemetry type: 'CoInterpreterTelemetry*';
		var: #telemetryEnabled type: 'int';
		var: #telemetryReceiverClass type: 'sqInt';
		var: #telemetrySendSelector type: 'sqInt'
]

{ #category : #generation }
CoInterpreterWithTelemetry class >> generate [
	| interpreterClass memoryManager compilerClass imageFormat imageReaderClassName imageWriterClassName options platformDirectory vmmaker |
	interpreterClass := CoInterpreterWithTelemetry.
	memoryManager := Spur64BitCoMemoryManager.
	compilerClass := StackToRegisterMappingCogitWithTelemetry.
	imageFormat := 'SpurFormat'.
	imageReaderClassName := SpurImageReader name.
	imageWriterClassName := SpurImageWriter name.
	options := #().

	VMMakerConfiguration initializeForPharo.

	platformDirectory := 'generated-vm-sources' asFileReference.
	platformDirectory ensureDeleteAll.

	[ vmmaker := VMMaker
			makerFor: interpreterClass
			and: compilerClass
			with: {#ObjectMemory.
					memoryManager name.
					#bytecodeTableInitializer.
					#initializeBytecodeTableForSistaV1.
					#ImageReader.
					imageReaderClassName.
					#ImageWriter.
					imageWriterClassName} , options
			to: platformDirectory
			platformDir: platformDirectory
			including: #()
			configuration: VMMakerConfiguration.

	vmmaker
		logger: NonInteractiveTranscript stdout;
		stopOnErrors: false;
		internal: #() external: #();
		generateInterpreterFile
		;generateCogitFiles ] valueSupplyingAnswer: true.

	^ platformDirectory
]

{ #category : #initialization }
CoInterpreterWithTelemetry class >> initialize [
	"execution happened in the interpreter"
	InterpreterCode := 1.
	"execution happened within machine code"
	MachineCode := 2.
	"execution happened at the boundary when transitioning from interpreter to machine code"
	InterpreterToMachineCode := 3.
	"execution happened at the boundary when transitioning from machine to interpreter code"
	MachineToInterpreterCode := 4.

	ReturnToInterpreterSignal := 1.
	ReturnToExecutiveSignal := 2.
	ReturnToMachineCodeFrameSignal := 3.
	MaybeReturnToMachineCodeFrameSignal := 4.
	ExternalAboutToReturnThroughSignal := 5.
	BaseFrameReturnSignal := 6.
	ReturnToExecutivePostContextWithSwitchSignal := 7.
	ReturnToExecutivePostContextWithoutSwitchSignal := 8.
	NonLocalReturnSignal := 9.
	CommonCallerReturnSignal := 10.
	SlowPrimitiveReturnSignal := 11.
	
	ArrowReturnTrampolineSignal := 21.
	BlockReturnTrampolineSignal := 22.
	EnilopmartReturnTrampolineSignal := 23.
	QuickPrimitiveReturnTrampolineSignal := 24.
	PrimReturnTrampolineSignal := 25
]

{ #category : #accessing }
CoInterpreterWithTelemetry class >> shouldGenerateTypedefFor: aStructClass [
	"Hack to work-around multiple definitions.  Sometimes a type has been defined in an include."
	^ (super shouldGenerateTypedefFor: aStructClass)
	  and: [ ({ CoInterpreterTelemetry } includes: aStructClass) not ]
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> activateCoggedNewMethod: inInterpreter [
	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalSend: 0
				toInstanceOf: 0
				isImmediate: false
				sourceId: 11
				framePointer: framePointer ].

	super activateCoggedNewMethod: inInterpreter
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> activateNewMethod [
	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalSend: 0
				toInstanceOf: 0
				isImmediate: false
				sourceId: 10
				framePointer: framePointer ].

	super activateNewMethod
]

{ #category : #accessing }
CoInterpreterWithTelemetry >> ceInterpretMethodFromPIC: aMethodObj receiver: rcvr [
	<api>
	| pic |
	<var: #pic type: #'CogMethod *'>
	
	telemetryEnabled
		ifTrue: [
			pic := self cCoerceSimple: self stackTop - cogit interpretOffset to: #'CogMethod *'.
			self
				emitSendSignal: pic selector
				receiver: rcvr
				sourceId: 3 ].
	
	super ceInterpretMethodFromPIC: aMethodObj receiver: rcvr
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> ceNonLocalReturn: returnValue [
	<api>
	<var: #returnValue type: #sqInt>
	<returnTypeC: #sqInt>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: NonLocalReturnSignal
				in: MachineCode
				framePointer: framePointer ].

	^ super ceNonLocalReturn: returnValue
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> ceReturnToInterpreter: anOop [
	"Perform a return from a machine code frame to an interpreted frame.
	 The machine code has executed a return instruction when the return address
	 is set to ceReturnToInterpreterPC.  Return the result and switch to the interpreter."

	"we return from machine code to interpreter"

	<api>
	"telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: ReturnToInterpreterSignal
				in: MachineToInterpreterCode
				framePointer: framePointer ]."

	super ceReturnToInterpreter: anOop
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> ceSend: selector above: methodClass to: rcvr numArgs: numArgs [
	<api>
	<option: #BytecodeSetHasDirectedSuperSend>
	<inline: false>
	
	telemetryEnabled
		ifTrue: [
			self
				emitSendSignal: selector
				receiver: rcvr
				sourceId: 6 ].
	
	super ceSend: selector above: methodClass to: rcvr numArgs: numArgs
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> ceSend: selector super: superNormalBar to: rcvr numArgs: numArgs [
	<api>
	| classTag classObj errSelIdx cogMethod |
	<inline: false>
	<var: #cogMethod type: #'CogMethod *'>
	<var: #newCogMethod type: #'CogMethod *'>
	"self printExternalHeadFrame"
	"self printStringOf: selector"
	
	telemetryEnabled
		ifTrue: [
			self
				emitSendSignal: selector
				receiver: rcvr
				sourceId: 7 ].
	
	"Entry-point for an unlinked send in a CogMethod.  Smalltalk stack looks like
					receiver
					args
		head sp ->	sender return pc
		
	If an MNU then defer to handleMNUInMachineCodeTo:... which will dispatch the MNU and
	may choose to allocate a closed PIC with a fast MNU dispatch for this send.  Otherwise
	attempt to link the send site as efficiently as possible.  All link attempts may fail; e.g.
	because we're out of code memory.

	Continue execution via either executeMethod or interpretMethodFromMachineCode:
	depending on whether the target method is cogged or not."
	
	cogit assertCStackWellAligned.
	self assert: (objectMemory addressCouldBeOop: rcvr).
	self sendBreakpoint: selector receiver: rcvr.
	superNormalBar = 0
		ifTrue: [classTag := objectMemory fetchClassTagOf: rcvr]
		ifFalse: [classTag := objectMemory classTagForClass: (self superclassOf: (self methodClassOf: (self frameMethodObject: framePointer)))].
	argumentCount := numArgs.
	(self lookupInMethodCacheSel: selector classTag: classTag)
		ifTrue:"check for coggability because method is in the cache"
			[self
				ifAppropriateCompileToNativeCode: newMethod
				selector: selector]
		ifFalse:
			[(objectMemory isOopForwarded: selector) ifTrue:
				[^self
					ceSend: (self handleForwardedSelectorFaultFor: selector)
					super: superNormalBar
					to: rcvr
					numArgs: numArgs].
			 (objectMemory isForwardedClassTag: classTag) ifTrue:
				[self assert: superNormalBar = 0.
				^self
					ceSend: selector
					super: superNormalBar
					to: (self handleForwardedSendFaultForReceiver: rcvr stackDelta: 1 "skip return pc")
					numArgs: numArgs].
			 messageSelector := selector.
			 classObj := objectMemory classForClassTag: classTag.
			 (errSelIdx := self lookupOrdinaryNoMNUEtcInClass: classObj) ~= 0 ifTrue:
				[(errSelIdx = SelectorDoesNotUnderstand
				  and: [(cogMethod := cogit cogMNUPICSelector: messageSelector
											receiver: rcvr
											methodOperand: (self mnuMethodOrNilFor: rcvr)
											numArgs: argumentCount) asUnsignedInteger
						> cogit minCogMethodAddress]) ifTrue:
						[cogit
							linkSendAt: (stackPages longAt: stackPointer)
							in: (self mframeHomeMethod: framePointer)
							to: cogMethod
							offset: (superNormalBar = 0
									ifTrue: [cogit entryOffset]
									ifFalse: [cogit noCheckEntryOffset])
							receiver: rcvr].
				telemetryEnabled
				ifTrue: [
					self print: 'About to send handleMNU:InMachineCodeTo:classForMessage'.
					self cr.
				].
				self handleMNU: errSelIdx InMachineCodeTo: rcvr classForMessage: classObj.
				self assert: false "NOTREACHED"]].
	"Method found and has a cog method.  Attempt to link to it.  The receiver's class may be young.
	 If the Cogit can't store young classes in inline caches we can link to an open PIC instead."
	(self maybeMethodHasCogMethod: newMethod) ifTrue:
		[cogMethod := self cogMethodOf: newMethod.
		 cogMethod selector = objectMemory nilObject
			ifTrue: [cogit setSelectorOf: cogMethod to: selector]
			ifFalse:
				["Deal with anonymous accessors, e.g. in Newspeak.  The cogMethod may not have the
				  correct selector.  If not, try and compile a new method with the correct selector."
				 cogMethod selector ~= selector ifTrue:
					[(cogit cog: newMethod selector: selector) ifNotNil:
						[:newCogMethod| cogMethod := newCogMethod]]].
		 cogMethod selector = selector
			ifTrue:
				[
				telemetryEnabled
					ifTrue: [
						self print: 'About to send linkSendAt:in:to::offset'.
						self cr.
				].
				cogit
					linkSendAt: (stackPages longAt: stackPointer)
					in: (self mframeHomeMethod: framePointer)
					to: cogMethod
					offset: (superNormalBar = 0
								ifTrue: [cogit entryOffset]
								ifFalse: [cogit noCheckEntryOffset])
					receiver: rcvr]
			ifFalse: "If patchToOpenPICFor:.. returns we're out of code memory"
				[superNormalBar = 0 ifTrue: "Open PICs perform normal sends. Can't patch if this is a super send."
					[
					telemetryEnabled
					ifTrue: [
						self print: 'About to send patchToOpenPICFor:numArgs:receiver'.
						self cr.
					].
					
					cogit
						patchToOpenPICFor: selector
						numArgs: numArgs
						receiver: rcvr]].
		 instructionPointer := self popStack.
			telemetryEnabled
				ifTrue: [
					 self print: 'About to send executeNewMethod'.
					 self cr.
				 ].
		 self executeNewMethod.
		 self assert: false "NOTREACHED"].
	instructionPointer := self popStack.
	telemetryEnabled
				ifTrue: [
					self print: 'About to send interpretMethodFromMachineCode'.
					self cr.
				].
	^self interpretMethodFromMachineCode.
	"NOTREACHED"
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> ceSendFromInLineCacheMiss: cogMethodOrPIC [
	<api>
	<var: #cogMethodOrPIC type: #'CogMethod *'>
	
	telemetryEnabled
		ifTrue: [
			| numArgs rcvr |
			
			numArgs := cogMethodOrPIC cmNumArgs.
			rcvr := self stackValue: numArgs + 1. "skip return pc"
			self
				emitSendSignal: cogMethodOrPIC selector
				receiver: rcvr
				sourceId: 2 ].
	
	super ceSendFromInLineCacheMiss: cogMethodOrPIC
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> commonCallerReturn [
	<returnTypeC: #void>
	"telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalReturnFrom: CommonCallerReturnSignal
				in: InterpreterCode
				framePointer: framePointer ]."

	super commonCallerReturn
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> commonSendOrdinary [
	telemetryEnabled
		ifTrue: [
			self
				emitSendSignal: messageSelector
				receiver: (self stackValue: argumentCount)
				sourceId: 5 ].

	super commonSendOrdinary
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithTelemetry >> disableTelemetry [
	<api>
	<inline: false>
	
	telemetryEnabled := false.
	telemetryReceiverClass := objectMemory nilObject.
	telemetrySendSelector := objectMemory nilObject.
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> doRecordContextSwitchTo: aNewProc [
	<inline: true>
	
	telemetryEnabled
		ifTrue: [
			| sched oldProc |
			sched := self schedulerPointer.
			oldProc := objectMemory fetchPointer: ActiveProcessIndex ofObject: sched.
			telemetry
				telemetrySignalContextSwitchFrom: oldProc
				to: aNewProc ]
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> emitReturnMachineSignalFromBaseFrame [
	<inline: true>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: BaseFrameReturnSignal
				in: MachineCode
				framePointer: framePointer	"(self frameSelector: framePointer)" ]
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> emitReturnStackSignalFromBaseFrame [
	<inline: true>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: BaseFrameReturnSignal
				in: MachineToInterpreterCode
				framePointer: framePointer	"(self frameSelector: framePointer)" ]
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> emitSendSignal: selector receiver: rcvr [
	<inline: true>

	self
		emitSendSignal: selector
		receiver: rcvr
		sourceId: 0
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> emitSendSignal: selector receiver: rcvr sourceId: sourceId [
	<inline: true>

	telemetryEnabled
		ifTrue: [
			(objectMemory isImmediate: rcvr)
				ifTrue: [
					telemetry
						telemetrySignalSend: selector
						toInstanceOf: rcvr
						isImmediate: true
						sourceId: sourceId
						framePointer: framePointer ]
				ifFalse: [
					telemetry
						telemetrySignalSend: selector
						toInstanceOf: (objectMemory fetchClassTagOf: rcvr)
						isImmediate: false
						sourceId: sourceId
						framePointer: framePointer ] ]
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithTelemetry >> enableTelemetry [
	<api>
	<inline: false>
	
	telemetryEnabled := true.
	telemetryReceiverClass := objectMemory nilObject.
	telemetrySendSelector := objectMemory nilObject.
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> executeNewMethod [
	telemetryEnabled
		ifTrue: [
			self print: 'About to emit Send from executeNewMethod'.
			self cr.

			telemetry
				telemetrySignalSend: 0
				toInstanceOf: 0
				isImmediate: false
				sourceId: 9
				framePointer: framePointer ].

	super executeNewMethod
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> executeNewMethod: eagerlyCompile [	
	"self emitSendSignal."
	super executeNewMethod: eagerlyCompile
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> externalAboutToReturn: resultOop through: aContext [
	<inline: true>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: ExternalAboutToReturnThroughSignal
				in: MachineCode
				framePointer: framePointer	"(self frameSelector: framePointer)" ].

	super externalAboutToReturn: resultOop through: aContext
]

{ #category : #accessing }
CoInterpreterWithTelemetry >> frameSelector: theFP [
	<var: #cogMethod type: #'CogMethod *'>
	<var: #theFP type: #'char *'>
	<var: #theMethod type: #'sqInt'>
	<returnTypeC: #'sqInt'>

	(self isMachineCodeFrame: theFP)
	 ifTrue: [
	 	| cogMethod |
	 	(self mframeIsBlockActivation: theFP)
	 		ifTrue: [ cogMethod := self mframeHomeMethod: theFP ]
	 		ifFalse: [ cogMethod := self mframeMethod: theFP ].
	 	^ cogMethod selector ]
	 ifFalse: [
	 	| theMethod |
	 	
	 	theMethod := self iframeMethod: theFP.
	 	^ self findSelectorOfMethod: theMethod ]
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> interpretMethodFromMachineCode [
	telemetryEnabled
		ifTrue: [
			self print: 'About to emit Send from interpretMethodFromMachineCode'.
			self cr.

			telemetry
				telemetrySignalSend: 0
				toInstanceOf: 0
				isImmediate: false
				sourceId: 8
				framePointer: framePointer ].

	super interpretMethodFromMachineCode
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> maybeReturnToMachineCodeFrame [
	super maybeReturnToMachineCodeFrame.
	"we return from interpreter to interpreter"
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: MaybeReturnToMachineCodeFrameSignal
				in: InterpreterCode
				framePointer: framePointer ]
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> return: returnValue toExecutive: inInterpreter [
	<var: #cogMethod type: #'CogMethod *'>
	telemetryEnabled
		ifTrue: [ (self isMachineCodeFrame: framePointer)
				ifTrue: [ telemetry
						telemetrySignalReturnFrom: ReturnToExecutiveSignal
						in: MachineCode
						framePointer: framePointer	"(self frameSelector: framePointer)" ]
				ifFalse: [ telemetry
						telemetrySignalReturnFrom: ReturnToExecutiveSignal
						in: InterpreterCode
						framePointer: framePointer	"(self frameSelector: framePointer)" ] ].

	super return: returnValue toExecutive: inInterpreter
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> returnToExecutive: inInterpreter postContextSwitch: switchedContext [
	"Return to the current frame, either by entering machine code, or longjmp-ing back to the
	 interpreter or simply returning, depending on where we are. To know whether to return or
	 enter machine code we have to know from whence we came.  We could have come from
	 the interpreter, either directly or via a machine code primitive.  We could have come from
	 machine code.  The instructionPointer tells us where from.  If it is above startOfMemory we're
	 in the interpreter.  If it is below, then we are in machine-code unless it is ceReturnToInterpreterPC,
	 in which case we're in a machine-code primitive called from the interpreter."

	"seems like it is not necessary to emit return here"

	<inline: false>
	<var: #cogMethod type: #'CogMethod *'>
	"telemetryEnabled
		ifTrue: [ | switchType |
			switchedContext
				ifTrue: [ switchType := ReturnToExecutivePostContextWithSwitchSignal ]
				ifFalse: [ switchType := ReturnToExecutivePostContextWithoutSwitchSignal ].

			(self isMachineCodeFrame: framePointer)
				ifTrue: [ telemetry
						telemetrySignalReturnFrom: switchType
						in: MachineCode
						framePointer: framePointer ]
				ifFalse: [ telemetry
						telemetrySignalReturnFrom: switchType
						in: InterpreterCode
						framePointer: framePointer  ] ]."

	super returnToExecutive: inInterpreter postContextSwitch: switchedContext
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> returnToMachineCodeFrame [
	"we return from interpreter to machine code"

	<inline: true>
	telemetryEnabled
		ifTrue: [ self
				flag: 'How do we detect a method to return too?
			I tried:
			 - self mframeCogMethod: framePointer
			 - self iframeMethod: framePointer.
			 - self mframeHomeMethod: framePointer.
			 - method'.
			telemetry
				telemetrySignalReturnFrom: ReturnToMachineCodeFrameSignal
				in: InterpreterToMachineCode
				framePointer: framePointer	"(self frameSelector: framePointer)" ].

	super returnToMachineCodeFrame
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> returnTopFromBlock [
	"Return top-of-stack to the caller of the current block activation."

	<returnTypeC: #void>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: 21
				in: InterpreterCode
				framePointer: framePointer	"(self frameSelector: framePointer)" ].

	localReturnValue := self stackTop.
	self commonCallerReturn
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> sendBreakpoint: selector receiver: rcvr [
	<inline: true>

	"self emitSendSignal: selector receiver: rcvr sourceId: 4."
	super sendBreakpoint: selector receiver: rcvr
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithTelemetry >> setTelemetry: interpreterTelemetry [
	<api>
	<inline: false>
	<var: #interpreterTelemetry type: #'CoInterpreterTelemetry *'>
	
	telemetry := interpreterTelemetry
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> slowPrimitiveResponse [
	| result |

	result := super slowPrimitiveResponse.
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: SlowPrimitiveReturnSignal
				in: InterpreterCode
				framePointer: framePointer ].
	^ result
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithTelemetry >> takeTelemetry [
	<api>
	<inline: false>
	<var: #prevTelemetry type: #'CoInterpreterTelemetry *'>
	| prevTelemetry |
	self disableTelemetry.
	prevTelemetry := telemetry.
	telemetry := 0.
	^ prevTelemetry
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> telemetryEmitBlockActivation [
	<api>
	<var: #cogMethod type: #'CogMethod *'>

	"telemetryEnabled
		ifTrue: [ | selector cogMethod |
			cogMethod := self mframeHomeMethod: framePointer.
			selector := cogMethod selector.

			selector = objectMemory nilObject
				ifTrue: [
					cogMethod cmType = CMMethod
						ifTrue: [ selector := self maybeSelectorOfMethod: cogMethod methodObject ] ].

			self
				emitSendSignal: selector
				receiver: (self frameReceiver: framePointer) ]"
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> telemetryEmitLinkedSend: theReceiver [
	| cogMethod |
	<api>
	<var: #cogMethod type: #'CogMethod *'>

	telemetryEnabled
		ifTrue: [
			cogMethod := self
				cCoerceSimple: (self stackTop - cogit traceLinkedSendOffset)
				to: #'CogMethod *'.

			self
				emitSendSignal: cogMethod selector
				receiver: theReceiver
				sourceId: 1 ]
]

{ #category : #accessing }
CoInterpreterWithTelemetry >> telemetryEnabledAddress [
	<api>
	<returnTypeC: #usqInt>

	^ (self addressOf: telemetryEnabled) asUnsignedInteger
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineEmitArrowReturn [
	<api>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: ArrowReturnTrampolineSignal
				in: MachineCode
				framePointer: framePointer ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineEmitBlockReturn [
	<api>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: BlockReturnTrampolineSignal
				in: MachineCode
				framePointer: framePointer ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineEmitEnilopmartReturn [
	<api>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: EnilopmartReturnTrampolineSignal
				in: MachineCode
				framePointer: framePointer ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineEmitPrimitiveActivation [
	<api>
	
	telemetryEnabled
		ifTrue: [ telemetry telemetrySignalPrimitiveActivation ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineEmitPrimitiveDeactivation [
	<api>
	
	telemetryEnabled
		ifTrue: [ telemetry telemetrySignalPrimitiveDeactivation ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineEmitPrimitiveMayCallMethods [
	<api>
	telemetryEnabled
		ifTrue: [ telemetry telemetrySignalPrimitiveMayCallMethods ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineEmitQuickPrimitiveReturn [
	<api>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: QuickPrimitiveReturnTrampolineSignal
				in: MachineCode
				framePointer: framePointer ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineInterpreterPrimitiveReturn [
	<api>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: InterpreterPrimitiveReturnTrampolineSignal
				in: MachineCode
				framePointer: framePointer ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineMachineCodePrimitiveReturn [
	<api>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: MachineCodePrimitiveReturnTrampolineSignal
				in: MachineCode
				framePointer: framePointer ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolinePrimReturn [
	<api>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: PrimReturnTrampolineSignal
				in: MachineCode
				framePointer: framePointer ]
]

{ #category : #'bytecode - context switch' }
CoInterpreterWithTelemetry >> transferTo: newProc from: sourceCode [
	self doRecordContextSwitchTo:newProc.
	super transferTo: newProc from: sourceCode
]
