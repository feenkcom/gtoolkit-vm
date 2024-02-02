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
		'telemetryMachineCodePrimitiveReturnTrampoline'
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
StackToRegisterMappingCogitWithTelemetry >> compilePrimitive [
	"quick primitives"
	
	primitiveIndex = 0
		ifTrue: [ ^ 0 ].
	
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimReturnTrampoline ].

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
	"backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimReturnTrampoline ]."
	
	^ super genPrimReturn
]

{ #category : #accessing }
StackToRegisterMappingCogitWithTelemetry >> genPrimitiveHashMultiply [
	"Implement 28-bit hashMultiply for SmallInteger and LargePositiveInteger receivers."
	| jmpFailImm jmpFailNonImm jmpNotSmallInt reenter |
	jmpNotSmallInt := objectRepresentation genJumpNotSmallInteger: ReceiverResultReg.

	objectRepresentation genConvertSmallIntegerToIntegerInReg: ReceiverResultReg.
	reenter :=
	self MoveCq: HashMultiplyConstant R: TempReg.
	self MulR: TempReg R: ReceiverResultReg.
	self AndCq: HashMultiplyMask R: ReceiverResultReg.
	objectRepresentation genConvertIntegerToSmallIntegerInReg: ReceiverResultReg.
	backEnd saveAndRestoreLinkRegAround: [ self CallRT: telemetryPrimReturnTrampoline ].
	self RetN: 0.

	jmpNotSmallInt jmpTarget: self Label.
	jmpFailImm := objectRepresentation genJumpImmediate: ReceiverResultReg.
	objectRepresentation genGetCompactClassIndexNonImmOf: ReceiverResultReg into: ClassReg.
	self CmpCq: ClassLargePositiveIntegerCompactIndex R: ClassReg.
	jmpFailNonImm := self JumpNonZero: 0.
	objectRepresentation genLoadSlot: 0 sourceReg: ReceiverResultReg destReg: ReceiverResultReg.
	self Jump: reenter.

	jmpFailImm jmpTarget: (jmpFailNonImm jmpTarget: self Label).
	^0
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
			
	telemetryInterpreterPrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineInterpreterPrimitiveReturn
			called: 'telemetryInterpreterPrimitiveReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.
			
	telemetryMachineCodePrimitiveReturnTrampoline :=
		self genTrampolineFor: #telemetryTrampolineMachineCodePrimitiveReturn
			called: 'telemetryMachineCodePrimitiveReturnTrampoline'
			arg: NoReg
			regsToSave: CallerSavedRegisterMask.

	telemetryBlockActivationTrampoline :=
		self genTrampolineFor: #telemetryEmitBlockActivation
			called: 'telemetryBlockActivationTrampoline'
			regsToSave: CallerSavedRegisterMask
]
