Extension { #name : #CogObjectRepresentation }

{ #category : #'*GToolkit-VMMaker-AddOns' }
CogObjectRepresentation >> genPrimitiveFullClosureValueWithTelemetry [
	"Defer to the cogit for this one, to match the split for genPrimitiveClosureValueWithTelemetry."
	<doNotGenerate>
	<option: #SistaV1BytecodeSet>
	^cogit genPrimitiveFullClosureValueWithTelemetry
]
