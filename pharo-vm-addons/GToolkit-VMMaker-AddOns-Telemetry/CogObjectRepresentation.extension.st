Extension { #name : #CogObjectRepresentation }

{ #category : #'*GToolkit-VMMaker-AddOns' }
CogObjectRepresentation >> genPrimitiveFullClosureValueWithTelemetry [
	"Defer to the cogit for this one, to match the split for genPrimitiveClosureValueWithTelemetry."
	<doNotGenerate>
	<option: #SistaV1BytecodeSet>
	^cogit genPrimitiveFullClosureValueWithTelemetry
]

{ #category : #'*GToolkit-VMMaker-AddOns' }
CogObjectRepresentation >> genPrimitiveMarkHandlerMethodWithTelemetry [
	<doNotGenerate>
	^cogit genPrimitiveMarkHandlerMethodWithTelemetry
]

{ #category : #'*GToolkit-VMMaker-AddOns' }
CogObjectRepresentation >> genPrimitiveMarkUnwindMethodWithTelemetry [
	<doNotGenerate>
	^cogit genPrimitiveMarkUnwindMethodWithTelemetry
]
