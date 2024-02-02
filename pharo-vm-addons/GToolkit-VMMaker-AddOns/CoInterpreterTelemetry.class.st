Class {
	#name : #CoInterpreterTelemetry,
	#superclass : #VMStructType,
	#instVars : [
		'payload',
		'sendFn',
		'returnFn',
		'contextSwitchFn',
		'primitiveActivationFn'
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
		value: 'contextSwitchFn' value: #('void (*' ')(void*, sqInt, sqInt)')
]

{ #category : #translation }
CoInterpreterTelemetry class >> structTypeName [
	^ self name allButFirst: 2 "Drop initial Co"
]

{ #category : #accessing }
CoInterpreterTelemetry >> contextSwitchFn [
	^ contextSwitchFn
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
