Extension { #name : #CogAbstractInstruction }

{ #category : #'*GToolkit-VMMaker-AddOns' }
CogAbstractInstruction >> compareQuickConstantByteSize [
	"See CogAbstractInstruction>>#dispatchConcretize to learn where to find byte sizes"
	self subclassResponsibility
]
