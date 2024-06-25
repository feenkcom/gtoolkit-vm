Extension { #name : #CoInterpreterPrimitives }

{ #category : #'*GToolkit-VMMaker-AddOns' }
CoInterpreterPrimitives >> primitiveCopyFromAtToAtSize [
	"primitiveCopyFrom: src at: srcStart to: dest at: destStart size: size"
	| size destStart destOop dest srcStart srcOop src |
	<export: true>
	
	"Amount of bytes to copy must not be negative"
	size := self stackIntegerValue: 0.
	size >= 0
		ifFalse: [ ^ self primitiveFailFor: PrimErrBadArgument ].

	"one-based start index of the destination"
	destStart := self stackIntegerValue: 1.
	destStart >= 1
		ifFalse: [ ^ self primitiveFailFor: PrimErrBadArgument ].
	
	destOop := self stackObjectValue: 2.

	dest := self
		maybeExternalAddressOf: (self stackObjectValue: 2)
		startingAt: destStart
		size: size.
	dest = 0
		ifTrue: [ ^ self primitiveFailFor: PrimErrBadArgument ].
	
	"An address already includes an offset, except when it is an ExternalAddress"
	(objectMemory fetchClassOf: destOop) = objectMemory classExternalAddress
		ifFalse: [ destStart := 1 ].
	
	"one-based start index of the source"
	srcStart := self stackIntegerValue: 3.
	srcStart >= 1
		ifFalse: [ ^ self primitiveFailFor: PrimErrBadArgument ].

	srcOop := self stackObjectValue: 4.
	src := self
		maybeExternalAddressOf: srcOop
		startingAt: srcStart
		size: size.
	src = 0
		ifTrue: [ ^ self primitiveFailFor: PrimErrBadArgument ].
	
	"An address already includes an offset, except when it is an ExternalAddress"
	(objectMemory fetchClassOf: srcOop) = objectMemory classExternalAddress
		ifFalse: [ srcStart := 1 ].

	self cCode: 'memcpy(dest + destStart - 1, src + srcStart - 1, size)'.
	self methodReturnInteger: size.
]
