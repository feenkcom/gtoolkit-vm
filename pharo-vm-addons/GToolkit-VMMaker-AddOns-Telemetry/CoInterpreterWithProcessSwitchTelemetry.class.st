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
