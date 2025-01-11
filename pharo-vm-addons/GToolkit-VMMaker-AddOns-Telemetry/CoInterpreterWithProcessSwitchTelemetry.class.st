Class {
	#name : #CoInterpreterWithProcessSwitchTelemetry,
	#superclass : #CoInterpreterPrimitives,
	#instVars : [
		'telemetry',
		'telemetryEnabled'
	],
	#category : #'GToolkit-VMMaker-AddOns-Telemetry'
}

{ #category : #'as yet unclassified' }
CoInterpreterWithProcessSwitchTelemetry class >> additionalHeadersDo: aBinaryBlock [
	"Evaluate aBinaryBlock with the names and contents of
	 any additional header files that need to be generated."
	 super additionalHeadersDo: aBinaryBlock.
	 
	aBinaryBlock
		value: 'telemetry.h'
		value: CoInterpreterTelemetry cogMethodHeader
]

{ #category : #'as yet unclassified' }
CoInterpreterWithProcessSwitchTelemetry class >> ancilliaryClasses [
	^ super ancilliaryClasses, { CoInterpreterTelemetry }
]

{ #category : #'as yet unclassified' }
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

{ #category : #'as yet unclassified' }
CoInterpreterWithProcessSwitchTelemetry class >> shouldGenerateTypedefFor: aStructClass [
	"Hack to work-around multiple definitions.  Sometimes a type has been defined in an include."
	^ (super shouldGenerateTypedefFor: aStructClass)
	  and: [ ({ CoInterpreterTelemetry } includes: aStructClass) not ]
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithProcessSwitchTelemetry >> disableTelemetry [
	<api>
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

{ #category : #'accessing - telemetry' }
CoInterpreterWithProcessSwitchTelemetry >> enableTelemetry [
	<api>
	<inline: false>
	
	telemetryEnabled := true
]

{ #category : #initialization }
CoInterpreterWithProcessSwitchTelemetry >> setTelemetry: interpreterTelemetry [
	<api>
	<inline: false>
	<var: #interpreterTelemetry type: #'CoInterpreterTelemetry *'>
	
	telemetry := interpreterTelemetry
]

{ #category : #'accessing - telemetry' }
CoInterpreterWithProcessSwitchTelemetry >> takeTelemetry [
	<api>
	<inline: false>
	<var: #prevTelemetry type: #'CoInterpreterTelemetry *'>
	| prevTelemetry |
	self disableTelemetry.
	prevTelemetry := telemetry.
	telemetry := 0.
	^ prevTelemetry
]

{ #category : #'as yet unclassified' }
CoInterpreterWithProcessSwitchTelemetry >> transferTo: newProc from: sourceCode [
	self doRecordContextSwitchTo:newProc.
	super transferTo: newProc from: sourceCode
]
