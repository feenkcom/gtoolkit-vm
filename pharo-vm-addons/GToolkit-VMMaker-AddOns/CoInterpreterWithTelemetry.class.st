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

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> ceBaseFrameReturn: returnValue [
	"Return across a page boundary.  The context to return to (which may be married)
	 is stored in the first word of the stack.  We get here when a return instruction jumps
	 to the ceBaseFrameReturn: address that is the return pc for base frames.  A consequence
	 of this is that the current frame is no longer valid since an interrupt may have overwritten
	 its state as soon as the stack pointer has been cut-back beyond the return pc.  So to have
	 a context to send the cannotReturn: message to we also store the base frame's context
	 in the second word of the stack page."
	<api>
	| contextToReturnTo contextToReturnFrom isAContext thePage newPage frameAbove |
	<var: #thePage type: #'StackPage *'>
	<var: #newPage type: #'StackPage *'>
	<var: #frameAbove type: #'char *'>
	self assert: (stackPages stackPageFor: stackPointer) = stackPage.
	self assert: stackPages mostRecentlyUsedPage = stackPage.
	cogit assertCStackWellAligned.
	self assert: framePointer = 0.
	self assert: stackPointer <= (stackPage baseAddress - objectMemory wordSize).
	self assert: stackPage baseFP + (2 * objectMemory wordSize) < stackPage baseAddress.
	"We would like to use the following assert but we can't since the stack pointer will be above the
	 base frame pointer in the base frame return and hence the 0 a base frame pointer points at could
	 be overwritten which will cause the isBaseFrame assert in frameCallerContext: to fail."
	"self assert: (self frameCallerContext: stackPage baseFP) = (stackPages longAt: stackPage baseAddress)."
	self assert: ((objectMemory addressCouldBeObj: (stackPages longAt: stackPage baseAddress - objectMemory wordSize))
				and: [objectMemory isContext: (stackPages longAt: stackPage baseAddress - objectMemory wordSize)]).
	contextToReturnTo := stackPages longAt: stackPage baseAddress.
	self assert: (objectMemory addressCouldBeObj: contextToReturnTo).

	"The stack page is effectively free now, so free it.  We must free it to be
	 correct in determining if contextToReturnTo is still married, and in case
	 makeBaseFrameFor: cogs a method, which may cause a code compaction,
	 in which case the frame must be free to avoid the relocation machinery
	 tracing the dead frame.  Since freeing now temporarily violates the page-list
	 ordering invariant, use the assert-free version."
	stackPages freeStackPageNoAssert: stackPage.
	isAContext := objectMemory isContext: contextToReturnTo.
	(isAContext
	 and: [self isStillMarriedContext: contextToReturnTo])
		ifTrue:
			[framePointer := self frameOfMarriedContext: contextToReturnTo.
			 thePage := stackPages stackPageFor: framePointer.
			 framePointer = thePage headFP
				ifTrue:
					[stackPointer := thePage headSP]
				ifFalse:
					["Returning to some interior frame, presumably because of a sender assignment.
					  Move the frames above to another page (they may be in use, e.g. via coroutining).
					  Make the interior frame the top frame."
					 frameAbove := self findFrameAbove: framePointer inPage: thePage.
					 "Since we've just deallocated a page we know that newStackPage won't deallocate an existing one."
					 newPage := stackPages newStackPage.
					 self assert: newPage = stackPage.
					 self moveFramesIn: thePage through: frameAbove toPage: newPage.
					 stackPages markStackPageMostRecentlyUsed: newPage.
					 self setStackPointersFromPage: thePage]]
		ifFalse:
			[(isAContext
			  and: [objectMemory isIntegerObject: (objectMemory fetchPointer: InstructionPointerIndex ofObject: contextToReturnTo)]) ifFalse:
				[contextToReturnFrom := stackPages longAt: stackPage baseAddress - objectMemory wordSize.
				 self tearDownAndRebuildFrameForCannotReturnBaseFrameReturnFrom: contextToReturnFrom
					to: contextToReturnTo
					returnValue: returnValue.
				^self externalCannotReturn: returnValue from: contextToReturnFrom].
			 "void the instructionPointer to stop it being incorrectly updated in a code
			 compaction in makeBaseFrameFor:."
			 instructionPointer := 0.
			 thePage := self makeBaseFrameFor: contextToReturnTo.
			 self setStackPointersFromPage: thePage].
	self setStackPageAndLimit: thePage.
	self assert: (stackPages stackPageFor: framePointer) = stackPage.
	(self isMachineCodeFrame: framePointer) ifTrue:
		[
		self emitReturnMachineSignalFromBaseFrame.
		self push: returnValue.
		 self callEnilopmart: #ceEnterCogCodePopReceiverReg.
		 "NOTREACHED"].
	instructionPointer := self stackTop.
	instructionPointer = cogit ceReturnToInterpreterPC ifTrue:
		[instructionPointer := self iframeSavedIP: framePointer].
	self setMethod: (self iframeMethod: framePointer).
	self emitReturnStackSignalFromBaseFrame.
	self stackTopPut: returnValue. "a.k.a. pop saved ip then push result"
	self assert: (self checkIsStillMarriedContext: contextToReturnTo currentFP: framePointer).
	self siglong: reenterInterpreter jmp: ReturnToInterpreter.
	"NOTREACHED"
	^nil
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
				framePointer: framePointer	"(self frameSelector: framePointer)" ].

	^ super ceNonLocalReturn: returnValue
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> ceReturnToInterpreter: anOop [
	"Perform a return from a machine code frame to an interpreted frame.
	 The machine code has executed a return instruction when the return address
	 is set to ceReturnToInterpreterPC.  Return the result and switch to the interpreter."

	"we return from machine code to interpreter"

	<api>
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: ReturnToInterpreterSignal
				in: MachineToInterpreterCode
				framePointer: framePointer	"(self frameSelector: framePointer)" ].

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
	<inline: false>
	
	telemetryEnabled
		ifTrue: [
			self
				emitSendSignal: selector
				receiver: rcvr
				sourceId: 7 ].
	
	super ceSend: selector super: superNormalBar to: rcvr numArgs: numArgs
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
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: CommonCallerReturnSignal
				in: InterpreterCode
				framePointer: framePointer ].

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
	"self emitSendSignal."
	super interpretMethodFromMachineCode
]

{ #category : #'bytecode - return' }
CoInterpreterWithTelemetry >> maybeReturnToMachineCodeFrame [
	super maybeReturnToMachineCodeFrame.	"we return from interpreter to interpreter"
	telemetryEnabled
		ifTrue: [ telemetry
				telemetrySignalReturnFrom: MaybeReturnToMachineCodeFrameSignal
				in: InterpreterCode
				framePointer: framePointer	"(self frameSelector: framePointer)" ]
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
	telemetryEnabled
		ifTrue: [ | switchType |
			switchedContext
				ifTrue: [ switchType := ReturnToExecutivePostContextWithSwitchSignal ]
				ifFalse: [ switchType := ReturnToExecutivePostContextWithoutSwitchSignal ].

			(self isMachineCodeFrame: framePointer)
				ifTrue: [ telemetry
						telemetrySignalReturnFrom: switchType
						in: MachineCode
						framePointer: framePointer	"(self frameSelector: framePointer)" ]
				ifFalse: [ telemetry
						telemetrySignalReturnFrom: switchType
						in: InterpreterCode
						framePointer: framePointer	"(self frameSelector: framePointer)" ] ].

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
