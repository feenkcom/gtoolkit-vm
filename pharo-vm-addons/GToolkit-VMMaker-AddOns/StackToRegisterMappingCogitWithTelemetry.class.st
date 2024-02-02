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
		'telemetryPrimReturnTrampoline'
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
	"quick primitives"
	endPC < initialPC ifTrue: [
		backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryQuickPrimitiveReturnTrampoline ].
		^0].
	
	^ super compileMethodBody
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genBlockReturn [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryBlockReturnTrampoline ].
	
	^ super genBlockReturn
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genEnilopmartReturn: forCall [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryEnilopmartReturnTrampoline ].
	
	^ super genEnilopmartReturn: forCall
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimReturn [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimReturnTrampoline ].
	
	^ super genPrimReturn
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genUpArrowReturn [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryArrowReturnTrampoline ].

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
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryBlockReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitBlockReturn
			called: 'telemetryBlockReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryQuickPrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitQuickPrimitiveReturn
			called: 'telemetryQuickPrimitiveReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryEnilopmartReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitEnilopmartReturn
			called: 'telemetryEnilopmartReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryPrimReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolinePrimReturn
			called: 'telemetryPrimReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryBlockActivationTrampoline :=
		self genTrampolineFor: #telemetryEmitBlockActivation
			called: 'telemetryBlockActivationTrampoline'
			regsToSave: CallerSavedRegisterMask
]
