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
		'telemetryPrimReturnTrampoline',
		'telemetryInterpreterPrimitiveReturnTrampoline',
		'telemetryMachineCodePrimitiveReturnTrampoline',
		'telemetryPrimitiveActivationTrampoline',
		'telemetryPrimitiveMayCallMethodsTrampoline'
	],
	#category : #'GToolkit-VMMaker-AddOns-Telemetry'
}

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry class >> declareCVarsIn: aCCodeGenerator [
	aCCodeGenerator
		addHeaderFileFirst: '"telemetry.h"'
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry class >> initializePrimitiveTable [
	| toOverride |
	super initializePrimitiveTable.
	
	toOverride := #(
		(198 genPrimitiveMarkUnwindMethodWithTelemetry)
		(199 genPrimitiveMarkHandlerMethodWithTelemetry)

		(207 genPrimitiveFullClosureValueWithTelemetry) "value[:value:value:value:] et al"
		(209 genPrimitiveFullClosureValueWithTelemetry) "valueNoContextSwitch[:value:] et al"
	).
	
	toOverride do: [ :each | primitiveTable at: each first put: nil ].
	
	self table: primitiveTable from: toOverride
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry class >> numTrampolines [
	^ super numTrampolines + (self instVarNames select: [ :each | each endsWith: #'Trampoline' ]) size
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
StackToRegisterMappingCogitWithTelemetry >> compilePrimitive [
	<inline: false>
	primitiveIndex = 0
		ifTrue: [ ^ 0 ].

	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimitiveActivationTrampoline ].

	^ super compilePrimitive
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
	<inline: true>
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimReturnTrampoline ].
	^ super genPrimReturn
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimitiveFullClosureValueWithTelemetry [
	| code |
	<option: #SistaV1BytecodeSet>
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimitiveMayCallMethodsTrampoline ].
	code := self genPrimitiveFullClosureValue.
	^ code
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimitiveMarkHandlerMethodWithTelemetry [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimitiveMayCallMethodsTrampoline ].
	^ self genFastPrimFail
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimitiveMarkUnwindMethodWithTelemetry [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimitiveMayCallMethodsTrampoline ].
	^ self genFastPrimFail
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimitivePerform [
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimitiveMayCallMethodsTrampoline ].
	^ super genPrimitivePerform
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
			regsToSave: CallerSavedRegisterMask.

	telemetryBlockReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitBlockReturn
			called: 'telemetryBlockReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.

	telemetryQuickPrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitQuickPrimitiveReturn
			called: 'telemetryQuickPrimitiveReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.

	telemetryEnilopmartReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitEnilopmartReturn
			called: 'telemetryEnilopmartReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.

	telemetryPrimReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolinePrimReturn
			called: 'telemetryPrimReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.
			
	telemetryInterpreterPrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineInterpreterPrimitiveReturn
			called: 'telemetryInterpreterPrimitiveReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.
			
	telemetryMachineCodePrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineMachineCodePrimitiveReturn
			called: 'telemetryMachineCodePrimitiveReturnTrampoline'
			regsToSave: CallerSavedRegisterMask.
			
	telemetryPrimitiveActivationTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitPrimitiveActivation
			called: 'telemetryPrimitiveActivationTrampoline'
			regsToSave: CallerSavedRegisterMask.
	
	telemetryPrimitiveMayCallMethodsTrampoline :=
		self genTrampolineFor: #telemetryTrampolineEmitPrimitiveMayCallMethods
			called: 'telemetryPrimitiveMayCallMethodsTrampoline'
			regsToSave: CallerSavedRegisterMask.


	telemetryBlockActivationTrampoline :=
		self genTrampolineFor: #telemetryEmitBlockActivation
			called: 'telemetryBlockActivationTrampoline'
			regsToSave: CallerSavedRegisterMask
]
