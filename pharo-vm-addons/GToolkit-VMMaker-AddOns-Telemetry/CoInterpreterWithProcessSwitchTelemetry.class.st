Class {
	#name : #CoInterpreterWithProcessSwitchTelemetry,
	#superclass : #CoInterpreterPrimitives,
	#instVars : [
		'telemetry',
		'telemetryEnabled'
	],
	#category : #'GToolkit-VMMaker-AddOns-Telemetry'
}

{ #category : #translation }
CoInterpreterWithProcessSwitchTelemetry class >> additionalHeadersDo: aBinaryBlock [
	"Evaluate aBinaryBlock with the names and contents of
	 any additional header files that need to be generated."
	 super additionalHeadersDo: aBinaryBlock.
	 
	aBinaryBlock
		value: 'telemetry.h'
		value: CoInterpreterTelemetry cogMethodHeader
]

{ #category : #translation }
CoInterpreterWithProcessSwitchTelemetry class >> ancilliaryClasses [
	^ super ancilliaryClasses, { CoInterpreterTelemetry }
]

{ #category : #translation }
CoInterpreterWithProcessSwitchTelemetry class >> declareCVarsIn: aCCodeGenerator [
	"Override to avoid repeating StackInterpreter's declarations and add our own extensions"
	self class == thisContext methodClass
		ifFalse: [^self]. "Don't duplicate decls in subclasses"

	aCCodeGenerator
		addHeaderFileFirst: '"telemetry.h"'.

	aCCodeGenerator
		var: #telemetry type: 'CoInterpreterTelemetry*';
		var: #telemetryEnabled type: 'int'
]

{ #category : #translation }
CoInterpreterWithProcessSwitchTelemetry class >> shouldGenerateTypedefFor: aStructClass [
	"Hack to work-around multiple definitions.  Sometimes a type has been defined in an include."
	^ (super shouldGenerateTypedefFor: aStructClass)
	  and: [ ({ CoInterpreterTelemetry } includes: aStructClass) not ]
]

{ #category : #'as yet unclassified' }
CoInterpreterWithProcessSwitchTelemetry >> checkForEventsMayContextSwitch: mayContextSwitch [
	"Check for possible interrupts and handle one if necessary.
	 Answer if a context switch has occurred."
	| switched sema now |
	<inline: false>
	<var: #now type: #usqLong>
	statCheckForEvents := statCheckForEvents + 1.

	"restore the stackLimit if it has been smashed."
	self restoreStackLimit.
	self writeBackHeadFramePointers.
	self assert: stackPage = stackPages mostRecentlyUsedPage.

	"Allow the platform to do anything it needs to do synchronously."
	self ioSynchronousCheckForEvents.

	self checkCogCompiledCodeCompactionCalledFor.

	objectMemory needGCFlag ifTrue:
		["sufficientSpaceAfterGC: runs the incremental GC and
		 then, if not enough space is available, the fullGC."
		 self doRecordGC.
		 (objectMemory sufficientSpaceAfterGC: 0) ifFalse:
			[self setSignalLowSpaceFlagAndSaveProcess]].

	mayContextSwitch ifFalse: [^false].

	switched := false.

	(profileProcess ~= objectMemory nilObject
	 or: [nextProfileTick > 0 and:[self ioHighResClock >= nextProfileTick]]) ifTrue:
		[nextProfileTick := 0.
		 "Take a sample (if not already done so) for the profiler if it is active.  This
		  must be done before any of the synchronousSignals below or else we will
		  attribute a pause in ioRelinquishProcessor to the newly activated process."
		 profileProcess = objectMemory nilObject ifTrue:
			[profileProcess := self activeProcess.
			 profileMethod := objectMemory nilObject].
		 "and signal the profiler semaphore if it is present"
		 (profileSemaphore ~= objectMemory nilObject
		  and: [self synchronousSignal: profileSemaphore]) ifTrue:
			[switched := true]].

	self checkDeliveryOfLongRunningPrimitiveSignal ifTrue:
		[switched := true].

	objectMemory signalLowSpace ifTrue:
		[objectMemory signalLowSpace: false. "reset flag"
		 sema := objectMemory splObj: TheLowSpaceSemaphore.
		 (sema ~= objectMemory nilObject
		  and: [self synchronousSignal: sema]) ifTrue:
			[switched := true]].

	"inIOProcessEvents prevents reentrancy into ioProcessEvents and allows disabling
	 ioProcessEvents e.g. for native GUIs.  We would like to manage that here but can't
	 since the platform code may choose to call ioProcessEvents itself in various places."
	
	(now := self ioUTCMicroseconds) >= nextPollUsecs ifTrue: [
		self ioProcessEvents. "sets interruptPending if interrupt key pressed; may callback"
		"msecs to wait before next call to ioProcessEvents."
		 nextPollUsecs := now + 20000
		 "Note that strictly speaking we might need to update 'now' at this point since
		 ioProcessEvents could take a very long time on some platforms" ].

	nextWakeupUsecs ~= 0 ifTrue:
		[now >= nextWakeupUsecs ifTrue:
			[nextWakeupUsecs := 0.
			 "set timer interrupt to 0 for 'no timer'"
			 sema := objectMemory splObj: TheTimerSemaphore.
			 (sema ~= objectMemory nilObject
			  and: [self synchronousSignal: sema]) ifTrue:
				[switched := true]]].

	"signal any pending finalizations"
	pendingFinalizationSignals > 0 ifTrue:
		[pendingFinalizationSignals := 0.
		 sema := objectMemory splObj: TheFinalizationSemaphore.
		 (sema ~= objectMemory nilObject
		  and: [self synchronousSignal: sema]) ifTrue:
			[switched := true]].

	"signal all semaphores in semaphoresToSignal"
	self signalExternalSemaphores ifTrue:
		[switched := true].

	^switched
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithProcessSwitchTelemetry >> disableTelemetry [
	<api>
	<export: true>
	<inline: false>
	
	telemetryEnabled := false.
]

{ #category : #'emitting signals' }
CoInterpreterWithProcessSwitchTelemetry >> doRecordContextSwitchTo: aNewProc [
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
CoInterpreterWithProcessSwitchTelemetry >> doRecordGC [
	<inline: true>
	
	telemetryEnabled
		ifTrue: [
			telemetry telemetrySignalBeginMachineMethod ]
]

{ #category : #'emitting signals' }
CoInterpreterWithProcessSwitchTelemetry >> doRecordSemaphoreWait: aSemaphore isLocked: isLocked [
	<inline: true>

	telemetryEnabled
		ifTrue: [
			telemetry
				telemetrySignalSemaphoreWait: aSemaphore
				process: self activeProcess
				isLocked: isLocked ]
]

{ #category : #'as yet unclassified' }
CoInterpreterWithProcessSwitchTelemetry >> doWaitSemaphore: sema reEnterInterpreter: hasToReenter [
	| excessSignals activeProc inInterpreter |
	<api>
	<returnTypeC: #void>	

	excessSignals := self fetchInteger: ExcessSignalsIndex ofObject: sema.
	excessSignals > 0
		ifTrue:
			[
			self doRecordSemaphoreWait: sema isLocked: false.
			self storeInteger: ExcessSignalsIndex
				ofObject: sema
				withValue: excessSignals - 1 ]
		ifFalse:
			["We're going to switch process, either to an interpreted frame or a machine
			  code frame. To know whether to return or enter machine code we have to
			  know from whence we came.  We could have come from the interpreter,
			  either directly or via a machine code primitive.  We could have come from
			  machine code.  The instructionPointer tells us where from:"
			self doRecordSemaphoreWait: sema isLocked: true.
			inInterpreter := self isInstructionPointerInInterpreter: instructionPointer.
			activeProc := self activeProcess.
			self addLastLink: activeProc toList: sema.
			self transferTo: self wakeHighestPriority from: CSWait.
			hasToReenter ifTrue: [self forProcessPrimitiveReturnToExecutivePostContextSwitch: inInterpreter]]
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithProcessSwitchTelemetry >> enableTelemetry [
	<api>
	<export: true>
	<inline: false>
	
	telemetryEnabled := true
]

{ #category : #initialization }
CoInterpreterWithProcessSwitchTelemetry >> setTelemetry: interpreterTelemetry [
	<api>
	<export: true>
	<inline: false>
	<var: #interpreterTelemetry type: #'CoInterpreterTelemetry *'>
	
	telemetry := interpreterTelemetry
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithProcessSwitchTelemetry >> takeTelemetry [
	<api>
	<export: true>
	<inline: false>
	<var: #prevTelemetry type: #'CoInterpreterTelemetry *'>
	| prevTelemetry |
	self disableTelemetry.
	prevTelemetry := telemetry.
	telemetry := 0.
	^ prevTelemetry
]

{ #category : #'process primitive support' }
CoInterpreterWithProcessSwitchTelemetry >> transferTo: newProc from: sourceCode [
	self doRecordContextSwitchTo:newProc.
	super transferTo: newProc from: sourceCode
]
