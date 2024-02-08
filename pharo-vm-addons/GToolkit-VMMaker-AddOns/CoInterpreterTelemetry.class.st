Class {
	#name : #CoInterpreterTelemetry,
	#superclass : #VMStructType,
	#instVars : [
		'payload',
		'sendFn',
		'returnFn',
		'contextSwitchFn',
		'primitiveActivationFn',
		'beginMachineMethodFn',
		'activateMachineMethodFn',
		'debugRecordClassFn'
	],
	#category : #'GToolkit-VMMaker-AddOns-Telemetry'
}

{ #category : #accessing }
CoInterpreterTelemetry class >> cogMethodHeader [
	^ String streamContents: [ :s | self printTypedefOn: s ]
]

{ #category : #translation }
CoInterpreterTelemetry class >> instVarNamesAndTypesForTranslationDo: aBinaryBlock [
	"enumerate aBinaryBlock with the names and C type strings for the inst vars to include in a struct of this type."

	aBinaryBlock
		value: 'payload' value: 'void*';
		value: 'sendFn' value: #('void (*' ')(void*, sqInt, sqInt, uint8_t, uint8_t, void*)');
		value: 'returnFn' value: #('void (*' ')(void*, uint8_t, uint8_t, void*)');
		value: 'primitiveActivationFn' value: #('void (*' ')(void*, uint8_t)');
		value: 'activateMachineMethodFn' value: #('void (*' ')(void*)');
		value: 'beginMachineMethodFn' value: #('void (*' ')(void*)');
		value: 'contextSwitchFn' value: #('void (*' ')(void*, sqInt, sqInt)');
		value: 'debugRecordClassFn' value: #('void (*' ')(void*, sqInt, uint8_t)')
]

{ #category : #translation }
CoInterpreterTelemetry class >> structTypeName [
	^ self name allButFirst: 2 "Drop initial Co"
]

{ #category : #accessing }
CoInterpreterTelemetry >> activateMachineMethodFn [
	^ activateMachineMethodFn
]

{ #category : #accessing }
CoInterpreterTelemetry >> beginMachineMethodFn [
	^ beginMachineMethodFn
]

{ #category : #accessing }
CoInterpreterTelemetry >> contextSwitchFn [
	^ contextSwitchFn
]

{ #category : #accessing }
CoInterpreterTelemetry >> debugRecordClassFn [
	^ debugRecordClassFn
]

{ #category : #accessing }
CoInterpreterTelemetry >> payload [
	^ payload
]

{ #category : #accessing }
CoInterpreterTelemetry >> primitiveActivationFn [
	^ primitiveActivationFn
]

{ #category : #accessing }
CoInterpreterTelemetry >> returnFn [
	^ returnFn
]

{ #category : #accessing }
CoInterpreterTelemetry >> sendFn [
	^ sendFn
]

{ #category : #signalling }
CoInterpreterTelemetry >> telemetrySignalActivateMachineMethod [
	"Is emitted when VM is about to activate a new machine method and begin its execution.
	Depending on the method and a VM state it will either perform a linked or unlinked method execution.
	Linked means that the execution will start from the method entry procedure while unlinked will skip it and will
	start directly from the method body."
	<inline: false>
	<returnTypeC:'void'>

	self
		perform: self activateMachineMethodFn
		with: self payload
]

{ #category : #signalling }
CoInterpreterTelemetry >> telemetrySignalBeginMachineMethod [
	"Is emitted when VM reached a method body of the method compiled to machine code."
	<inline: false>
	<returnTypeC:'void'>

	self
		perform: self beginMachineMethodFn
		with: self payload
]

{ #category : #signalling }
CoInterpreterTelemetry >> telemetrySignalContextSwitchFrom: anOldProc to: aNewProc [
	<inline: false>
	<returnTypeC:'void'>
	
	self
		perform: self contextSwitchFn
		with: self payload
		with: anOldProc
		with: aNewProc
]

{ #category : #signalling }
CoInterpreterTelemetry >> telemetrySignalDebugClass: aClass isImmediate: isImmediate [
	<inline: false>
	<returnTypeC:'void'>

	self
		perform: self debugRecordClassFn
		with: self payload
		with: aClass
		with: isImmediate
]

{ #category : #signalling }
CoInterpreterTelemetry >> telemetrySignalPrimitiveActivation [
	<inline: false>
	<returnTypeC:'void'>
	
	self
		perform: self primitiveActivationFn
		with: self payload
		with: 1
]

{ #category : #signalling }
CoInterpreterTelemetry >> telemetrySignalPrimitiveDeactivation [
	<inline: false>
	<returnTypeC:'void'>
	
	self
		perform: self primitiveActivationFn
		with: self payload
		with: 0
]

{ #category : #signalling }
CoInterpreterTelemetry >> telemetrySignalPrimitiveMayCallMethods [
	<inline: false>
	<returnTypeC:'void'>
	
	self
		perform: self primitiveActivationFn
		with: self payload
		with: 2
]

{ #category : #signalling }
CoInterpreterTelemetry >> telemetrySignalReturnFrom: sourceId in: executionLocation framePointer: aFP [
	<inline: false>
	<var: #sourceId type: 'uint8_t'>
	<var: #executionLocation type: 'uint8_t'>
	<returnTypeC:'void'>
	
	self
		perform: self returnFn
		with: self payload
		with: sourceId
		with: executionLocation
		with: aFP
]

{ #category : #signalling }
CoInterpreterTelemetry >> telemetrySignalSend: aSelector toInstanceOf: aClass isImmediate: isImmediate sourceId: sourceId framePointer: aFP [
	<inline: false>
	<returnTypeC:'void'>

	self
		perform: self sendFn
		with: self payload
		with: aClass
		with: aSelector
		with: isImmediate
		with: sourceId
		with: aFP
]
