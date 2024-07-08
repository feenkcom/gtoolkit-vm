Extension { #name : #StackInterpreterPrimitives }

{ #category : #'*GToolkit-VMMaker-AddOns' }
StackInterpreterPrimitives >> cAddressOf: rcvr startingAt: byteOffset size: byteSize [
	"Return an int of the address of the byteSize slot (byte, short, int, whatever) at byteOffset in rcvr.
	If the rcvr is an ExternalAddress the initial address is its contents. If the receiver is any other byte/word array, the address is inside the array"
	| rcvrClass rcvrSize addr |

	((objectMemory isBytes: rcvr) or: [ objectMemory isWords: rcvr ]) ifFalse:
		[ ^self primitiveFail].
	(byteOffset > 0) ifFalse:
		[ ^ self primitiveFail].
	
	rcvrClass := objectMemory fetchClassOf: rcvr.
	rcvrSize := objectMemory byteSizeOf: rcvr.
	self cCode: 'fprintf(stderr, "cAddressOf: rcvrSize: %ld\n", rcvrSize)'.

	rcvrClass = objectMemory classExternalAddress ifTrue:[
		rcvrSize = BytesPerWord ifFalse:
			[ ^ self primitiveFail].
		
		addr := objectMemory fetchPointer: 0 ofObject: rcvr.
		addr = 0 ifTrue: 
			[ ^ 0 ].
	] ifFalse:[
		(byteOffset+byteSize-1 <= rcvrSize) ifFalse:
			[ ^ 0].
		addr := self cCoerce: (objectMemory firstIndexableField: rcvr) to: #'sqIntptr_t'.
	].
	addr := addr + byteOffset - 1.
	^ addr
]
