Class {
	#name : #StackToRegisterMappingCogitWithTelemetry,
	#superclass : #StackToRegisterMappingCogit,
	#instVars : [
		'telemetryLinkedSendTrampoline',
		'telemetryBlockActivationTrampoline',
		'telemetryLinkedReturnTrampoline'
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
StackToRegisterMappingCogitWithTelemetry >> genBlockReturn [
	super genBlockReturn.
	
	"emit signal after framePointer is set"
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryLinkedReturnTrampoline ].

	^ 0
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genUpArrowReturn [
	"We should emit return signal only after the framePointer is set. Therefore
		- when returning from block emit the signal before generator
		- and after when returning from a method"
	inBlock > 0 ifTrue: [
		backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryLinkedReturnTrampoline ] ].

	super genUpArrowReturn.

	inBlock > 0 ifFalse: [
		backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryLinkedReturnTrampoline ] ].

	^ 0
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> generateTracingTrampolines [
	super generateTracingTrampolines.
	
	telemetryLinkedSendTrampoline :=
		self genTrampolineFor: #telemetryEmitLinkedSend:
			called: 'telemetryLinkedSendTrampoline'
			arg: ReceiverResultReg
			regsToSave: CallerSavedRegisterMask.
			
	telemetryLinkedReturnTrampoline :=
		self genTrampolineFor: #telemetryEmitLinkedReturn
			called: 'telemetryLinkedReturnTrampoline'
			arg: ReceiverResultReg
			regsToSave: CallerSavedRegisterMask.

	telemetryBlockActivationTrampoline :=
		self genTrampolineFor: #telemetryEmitBlockActivation
			called: 'telemetryBlockActivationTrampoline'
			regsToSave: CallerSavedRegisterMask
]
