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
		'telemetrySendSelector',
		'skipNextOrdinarySend'
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
		'ReturnToExecutiveInInterpreterSignal',
		'ReturnToExecutiveInMachineCodeSignal',
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
	PrimReturnTrampolineSignal := 25.
	
	ReturnToExecutiveInInterpreterSignal := 31.
	ReturnToExecutiveInMachineCodeSignal := 32.
]

{ #category : #accessing }
CoInterpreterWithTelemetry class >> shouldGenerateTypedefFor: aStructClass [
	"Hack to work-around multiple definitions.  Sometimes a type has been defined in an include."
	^ (super shouldGenerateTypedefFor: aStructClass)
	  and: [ ({ CoInterpreterTelemetry } includes: aStructClass) not ]
]

{ #category : #'bytecode - send machine code' }
CoInterpreterWithTelemetry >> activateCoggedNewMethod: inInterpreter [
	telemetryEnabled
		ifTrue: [ telemetry telemetrySignalActivateMachineMethod ].

	super activateCoggedNewMethod: inInterpreter
]

{ #category : #'bytecode - send interpreter' }
CoInterpreterWithTelemetry >> bytecodePrimValue [
	telemetryEnabled
		ifTrue: [
			self
				emitSpecialSendSignal: 25
				receiver: self stackTop ].

	super bytecodePrimValue
]

{ #category : #'bytecode - send interpreter' }
CoInterpreterWithTelemetry >> bytecodePrimValueWithArg [
	telemetryEnabled
		ifTrue: [
			self
				emitSpecialSendSignal: 26
				receiver: (self stackValue: 1) ].

	super bytecodePrimValueWithArg
]

{ #category : #'bytecode - send machine code' }
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

{ #category : #'bytecode - send machine code' }
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

{ #category : #'bytecode - send machine code' }
CoInterpreterWithTelemetry >> ceSend: selector super: superNormalBar to: rcvr numArgs: numArgs [
	<api>
	<inline: false>
	
	telemetryEnabled
		ifTrue: [
			self
				emitSendSignal: selector
				receiver: rcvr
				sourceId: 7 ].
	
	super ceSend: selector super: superNormalBar to: rcvr numArgs: numArgs
]

{ #category : #'bytecode - send machine code' }
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

{ #category : #'bytecode - send interpreter' }
CoInterpreterWithTelemetry >> commonSendOrdinary [
	telemetryEnabled
		ifTrue: [
			skipNextOrdinarySend
				ifTrue: [ skipNextOrdinarySend := false ]
				ifFalse: [
					self
						emitSendSignal: messageSelector
						receiver: (self stackValue: argumentCount)
						sourceId: 5 ] ].

	super commonSendOrdinary
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithTelemetry >> disableTelemetry [
	<api>
	<inline: false>
	
	telemetryEnabled := false.
	telemetryReceiverClass := objectMemory nilObject.
	telemetrySendSelector := objectMemory nilObject.
	skipNextOrdinarySend := false
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

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> emitReturnMachineSignalFromBaseFrame [
	<inline: true>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: BaseFrameReturnSignal
				in: MachineCode
				framePointer: framePointer	"(self frameSelector: framePointer)" ]
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> emitReturnStackSignalFromBaseFrame [
	<inline: true>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: BaseFrameReturnSignal
				in: MachineToInterpreterCode
				framePointer: framePointer	"(self frameSelector: framePointer)" ]
]

{ #category : #'emitting signals' }
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

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> emitSpecialSendSignal: selectorIndex receiver: rcvr [
	<inline: true>

	self
		emitSendSignal: (self specialSelector: selectorIndex)
		receiver: rcvr
		sourceId: 22.
	skipNextOrdinarySend := true
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithTelemetry >> enableTelemetry [
	<api>
	<inline: false>
	
	telemetryEnabled := true.
	telemetryReceiverClass := objectMemory nilObject.
	telemetrySendSelector := objectMemory nilObject.
	skipNextOrdinarySend := false
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> executeNewMethod [
	
	"Shouldn't be used for collecting telemetry signals, only for debugging purposes"
	
	"telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalSend: (objectMemory findSelectorOfMethod: newMethod)
				toInstanceOf: 0
				isImmediate: false
				sourceId: 9
				framePointer: framePointer ]."

	super executeNewMethod
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> executeQuickPrimitive [

	self flag: 'Should it be used?'.

	"telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalSend: 0
				toInstanceOf: 0
				isImmediate: false
				sourceId: 14
				framePointer: framePointer ]."

	super executeQuickPrimitive.
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> externalAboutToReturn: resultOop through: aContext [
	<inline: true>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: ExternalAboutToReturnThroughSignal
				in: MachineCode
				framePointer: framePointer ].

	super externalAboutToReturn: resultOop through: aContext
]

{ #category : #'bytecode - send interpreter' }
CoInterpreterWithTelemetry >> fetchNextBytecode [
	telemetryEnabled
		ifTrue: [ skipNextOrdinarySend := false ].

	super fetchNextBytecode
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> interpretMethodFromMachineCode [

	"Shouldn't be used for collecting telemetry signals, only for debugging purposes"

	"telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalSend: (objectMemory findSelectorOfMethod: newMethod)
				toInstanceOf: 0
				isImmediate: false
				sourceId: 8
				framePointer: framePointer ]."

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
		ifTrue: [
			| aSource |
			
			inInterpreter
				ifTrue: [ aSource := ReturnToExecutiveInInterpreterSignal ]
				ifFalse: [ aSource := ReturnToExecutiveInMachineCodeSignal ].
			
			(self isMachineCodeFrame: framePointer)
				ifTrue: [
					telemetry
						telemetrySignalReturnFrom: aSource
						in: MachineCode
						framePointer: framePointer ]
				ifFalse: [
					telemetry
						telemetrySignalReturnFrom: aSource
						in: InterpreterCode
						framePointer: framePointer ] ].

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
	"telemetryEnabled
		ifTrue: [ | switchType |
			switchedContext
				ifTrue: [ switchType := ReturnToExecutivePostContextWithSwitchSignal ]
				ifFalse: [ switchType := ReturnToExecutivePostContextWithoutSwitchSignal ].

			(self isMachineCodeFrame: framePointer)
				ifTrue: [
					telemetry
						telemetrySignalSend: 0
						toInstanceOf: 0
						isImmediate: false
						sourceId: 12
						framePointer: framePointer ]
				ifFalse: [
					telemetry
						telemetrySignalSend: 0
						toInstanceOf: 0
						isImmediate: false
						sourceId: 13
						framePointer: framePointer ] ]."

	super returnToExecutive: inInterpreter postContextSwitch: switchedContext
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> returnToMachineCodeFrame [
	"we return from interpreter to machine code"

	<inline: true>
	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalReturnFrom: ReturnToMachineCodeFrameSignal
				in: InterpreterToMachineCode
				framePointer: framePointer ].

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
				framePointer: framePointer ].

	localReturnValue := self stackTop.
	self commonCallerReturn
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

	"telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalSend: 0
				toInstanceOf: 0
				isImmediate: false
				sourceId: 16
				framePointer: framePointer ]."
	
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

{ #category : #'emitting signals' }
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

{ #category : #accessing }
CoInterpreterWithTelemetry >> telemetryEnabledAddress [
	<api>
	<returnTypeC: #usqInt>

	^ (self addressOf: telemetryEnabled) asUnsignedInteger
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineEmitArrowReturn [
	<api>
	telemetry
		telemetrySignalReturnFrom: ArrowReturnTrampolineSignal
		in: MachineCode
		framePointer: framePointer
]

{ #category : #'emitting signals' }
CoInterpreterWithTelemetry >> telemetryTrampolineEmitBeginMachineMethod [
	<api>
	"Trampoline will not be executed unless telemetry is enabled.
	There is no need to check it again here"
	
	telemetry telemetrySignalBeginMachineMethod
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
CoInterpreterWithTelemetry >> telemetryTrampolineEmitLinkedSend: theReceiver [
	| cogMethod |
	<api>
	<var: #cogMethod type: #'CogMethod *'>

	cogMethod := self
		cCoerceSimple: (self stackTop - cogit traceLinkedSendOffset)
		to: #'CogMethod *'.

	self
		emitSendSignal: cogMethod selector
		receiver: theReceiver
		sourceId: 1
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
CoInterpreterWithTelemetry >> telemetryTrampolineEmitSpecialSelectorSend: theReceiver [
	<api>
	<var: #cogMethod type: #'CogMethod *'>

	"self
		emitSendSignal: 0
		receiver: theReceiver
		sourceId: 21"
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
