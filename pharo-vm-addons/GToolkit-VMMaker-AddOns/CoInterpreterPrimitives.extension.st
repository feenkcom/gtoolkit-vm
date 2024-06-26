Extension { #name : #CoInterpreterPrimitives }

{ #category : #'*GToolkit-VMMaker-AddOns' }
CoInterpreterPrimitives >> primitiveCopyFromAtToAtSize [
	"primitiveCopyFrom: src at: srcStart to: dest at: destStart size: size"
	| size destStart dest srcStart src |
	<export: true>
	
	"Amount of bytes to copy must not be negative"
	size := self stackIntegerValue: 0.
	size >= 0
		ifFalse: [ ^ self primitiveFailFor: PrimErrBadArgument ].

	"one-based start index of the destination"
	destStart := self stackIntegerValue: 1.
	destStart >= 1
		ifFalse: [ ^ self primitiveFailFor: PrimErrBadArgument ].
	
	dest := self
		maybeExternalAddressOf: (self stackObjectValue: 2)
		startingAt: destStart
		size: size.
	dest = 0
		ifTrue: [ ^ self primitiveFailFor: PrimErrBadArgument ].

	"one-based start index of the source"
	srcStart := self stackIntegerValue: 3.
	srcStart >= 1
		ifFalse: [ ^ self primitiveFailFor: PrimErrBadArgument ].

	src := self
		maybeExternalAddressOf: (self stackObjectValue: 4)
		startingAt: srcStart
		size: size.
	src = 0
		ifTrue: [ ^ self primitiveFailFor: PrimErrBadArgument ].

	self cCode: 'memcpy(dest, src, size)'.
	self methodReturnInteger: size.
]
