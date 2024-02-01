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
StackToRegisterMappingCogitWithTelemetry >> compileMethodBody [
	<inline: true>
	| result |
	"Compile the top-level method body."
	endPC < initialPC ifTrue: [ ^ 0 ].	"quick primitives"	"When compiling, skip any initial CallPrimitive and optional StorePrimErrCode bytecodes.
	 These are dealt with in compileFrameBuild."
	result := self
			compileAbstractInstructionsFrom: initialPC
					+ (self deltaToSkipPrimAndErrorStoreIn: methodObj header: methodHeader)
			through: endPC.
	backEnd
		saveAndRestoreLinkRegAround: [ self CallRT: telemetryLinkedReturnTrampoline ].
	^ result
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
