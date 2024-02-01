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
		'BaseFrameReturnSignal',
		'ExternalAboutToReturnThroughSignal',
		'InterpreterCode',
		'InterpreterToMachineCode',
		'MachineCode',
		'MachineToInterpreterCode',
		'MaybeReturnToMachineCodeFrameSignal',
		'NonLocalReturnSignal',
		'ReturnToExecutivePostContextWithSwitchSignal',
		'ReturnToExecutivePostContextWithoutSwitchSignal',
		'ReturnToExecutiveSignal',
		'ReturnToInterpreterSignal',
		'ReturnToMachineCodeFrameSignal'
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
	NonLocalReturnSignal := 9
]

{ #category : #accessing }
CoInterpreterWithTelemetry class >> shouldGenerateTypedefFor: aStructClass [
	"Hack to work-around multiple definitions.  Sometimes a type has been defined in an include."
	^ (super shouldGenerateTypedefFor: aStructClass)
	  and: [ ({ CoInterpreterTelemetry } includes: aStructClass) not ]
]

{ #category : #accessing }
CoInterpreterWithTelemetry >> activateNewFullClosure: blockClosure method: theMethod numArgs: numArgs mayContextSwitch: mayContextSwitch [
	"telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalSend: 0
				toInstanceOf: 0
				isImmediate: 0
				framePointer: framePointer ]."
	super activateNewFullClosure: blockClosure method: theMethod numArgs: numArgs mayContextSwitch: mayContextSwitch
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
	
	telemetryEnabled ifTrue: [
		telemetry
			telemetrySignalReturn: objectMemory nilObject
			from: BaseFrameReturnSignal
			in: MachineCode
			framePointer: framePointer ].

	super ceNonLocalReturn: returnValue
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> ceReturnToInterpreter: anOop [
	"Perform a return from a machine code frame to an interpreted frame.
	 The machine code has executed a return instruction when the return address
	 is set to ceReturnToInterpreterPC.  Return the result and switch to the interpreter."
	<api>

	"we return from machine code to interpreter"
	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalReturn: (self iframeMethod: framePointer)
				from: ReturnToInterpreterSignal
				in: MachineToInterpreterCode
				framePointer: framePointer ].

	super ceReturnToInterpreter: anOop
]

{ #category : #accessing }
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

{ #category : #accessing }
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

{ #category : #accessing }
CoInterpreterWithTelemetry >> commonCallerReturn [
	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalReturn: objectMemory nilObject
				from: 20
				in: InterpreterCode
				framePointer: framePointer ].

	super commonCallerReturn
]

{ #category : #accessing }
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
	<var: #cogMethod type: #'CogMethod *'>
	
	telemetryEnabled ifTrue: [
		| cogMethod |
		cogMethod := self mframeCogMethod: framePointer.
		telemetry
			telemetrySignalReturn: cogMethod methodObject
			from: BaseFrameReturnSignal
			in: MachineCode
			framePointer: framePointer ]
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> emitReturnStackSignalFromBaseFrame [
	telemetryEnabled ifTrue: [
		telemetry
			telemetrySignalReturn: (self iframeMethod: framePointer)
			from: BaseFrameReturnSignal
			in: MachineToInterpreterCode
			framePointer: framePointer ]
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
CoInterpreterWithTelemetry >> executeNewMethod: eagerlyCompile [	
	"self emitSendSignal."
	super executeNewMethod: eagerlyCompile
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> externalAboutToReturn: resultOop through: aContext [
	<inline: true>
	
	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalReturn: objectMemory nilObject
				from: ExternalAboutToReturnThroughSignal
				in: MachineCode
				framePointer: framePointer ].
	
	super externalAboutToReturn: resultOop through: aContext
]

{ #category : #'bytecode - send' }
CoInterpreterWithTelemetry >> interpretMethodFromMachineCode [
	"self emitSendSignal."
	super interpretMethodFromMachineCode
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> maybeReturnToMachineCodeFrame [
	super maybeReturnToMachineCodeFrame.
	
	"we return from interpreter to interpreter"
	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalReturn: (self iframeMethod: framePointer)
				from: MaybeReturnToMachineCodeFrameSignal
				in: InterpreterCode
				framePointer: framePointer ]
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> return: returnValue toExecutive: inInterpreter [
	<var: #cogMethod type: #'CogMethod *'>

	telemetryEnabled ifTrue: [
		(self isMachineCodeFrame: framePointer)
			ifTrue: [
				| cogMethod |
				cogMethod := self mframeCogMethod: framePointer.
				telemetry
					telemetrySignalReturn: cogMethod methodObject
					from: ReturnToExecutiveSignal
					in: MachineCode
					framePointer: framePointer ]
			ifFalse: [
				telemetry
					telemetrySignalReturn: (self iframeMethod: framePointer)
					from: ReturnToExecutiveSignal
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
	<inline: false>
	<var: #cogMethod type: #'CogMethod *'>
	
	"seems like it is not necessary to emit return here"
	
	"telemetryEnabled ifTrue: [
		| switchType |
		
		switchedContext
			ifTrue: [ switchType := ReturnToExecutivePostContextWithSwitchSignal ]
			ifFalse: [ switchType := ReturnToExecutivePostContextWithoutSwitchSignal ].
		
		(self isMachineCodeFrame: framePointer)
			ifTrue: [
				| cogMethod |
				
				cogMethod := self mframeCogMethod: framePointer.
				telemetry
					telemetrySignalReturn: cogMethod methodObject
					from: switchType
					in: MachineCode
					framePointer: framePointer ]
			ifFalse: [
				telemetry
					telemetrySignalReturn: (self iframeMethod: framePointer)
					from: switchType
					in: InterpreterCode
					framePointer: framePointer ] ]."

	super returnToExecutive: inInterpreter postContextSwitch: switchedContext
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> returnToMachineCodeFrame [
	<inline: true>
	
	"we return from interpreter to machine code"
	telemetryEnabled
		ifTrue: [
			self flag: 'How do we detect a method to return too?
			I tried:
			 - self mframeCogMethod: framePointer
			 - self iframeMethod: framePointer.
			 - self mframeHomeMethod: framePointer.
			 - method'.
			telemetry
				telemetrySignalReturn: objectMemory nilObject
				from: ReturnToMachineCodeFrameSignal
				in: InterpreterToMachineCode
				framePointer: framePointer ].

	super returnToMachineCodeFrame
]

{ #category : #accessing }
CoInterpreterWithTelemetry >> returnTopFromBlock [
	"Return top-of-stack to the caller of the current block activation."

	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalReturn: objectMemory nilObject
				from: 21
				in: InterpreterCode
				framePointer: framePointer ].

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
CoInterpreterWithTelemetry >> telemetryEmitLinkedReturn: aFramePointer [
	<api>

	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalReturn: objectMemory nilObject
				from: 22
				in: MachineCode
				framePointer: aFramePointer ]
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

{ #category : #'bytecode - context switch' }
CoInterpreterWithTelemetry >> transferTo: newProc from: sourceCode [
	self doRecordContextSwitchTo:newProc.
	super transferTo: newProc from: sourceCode
]
