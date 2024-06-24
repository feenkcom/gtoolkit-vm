Extension { #name : #CoInterpreterPrimitives }

{ #category : #'*GToolkit-VMMaker-AddOns' }
CoInterpreterPrimitives >> primitiveCopyFromByteArrayToExternalAddress [
	"primitiveCopy: aByteArray startingAt: start size: size toExternal: anExternalAddress"
	| oop externalPtr externalAddress size baPtr start baSize |
	<export: true>
	<var: 'externalPtr' type: #'void **'>
	<var: 'externalAddress' type: #'void *'>
	<var: 'baPtr' type: #'void *'>

	oop := self stackObjectValue: 0.
	(self failed
		or: [(self is: oop KindOf: 'ExternalAddress') not]) ifTrue:
			[^self primitiveFailForOSError: 1 ].
	externalPtr := self arrayValueOf: oop.
	externalAddress := self cCode: '*externalPtr'.
	externalAddress = 0 ifTrue:
			[^self primitiveFailForOSError: 2 ].

	size := self stackIntegerValue: 1.
	(self failed or: [ size < 0 ]) ifTrue:
		[^self primitiveFailForOSError: 3 ].

	start := self stackIntegerValue: 2.
	(self failed or: [ start < 1 ]) ifTrue:
			[^self primitiveFailForOSError: 4 ].

	oop := self stackObjectValue: 3.
	(self failed
		or: [(self is: oop KindOf: 'ByteArray') not]) ifTrue:
			[^self primitiveFailForOSError: 5 ].
	baPtr := self arrayValueOf: oop.
	baSize := self stSizeOf: oop.

	(start + size - 1 > baSize) ifTrue:
		[ ^self primitiveFailForOSError: 6 ].

	baPtr := baPtr + start - 1.

	self cCode: 'memcpy(externalAddress, baPtr, size)'.
	self methodReturnInteger: size.
]

{ #category : #'*GToolkit-VMMaker-AddOns' }
CoInterpreterPrimitives >> primitiveCopyFromExternalAddressToByteArray [
	"primitiveCopy: anExternalAddress size: size to: aByteArray startingAt: start"
	| oop externalPtr externalAddress size baPtr start baSize |
	<export: true>
	<var: 'externalPtr' type: #'void **'>
	<var: 'externalAddress' type: #'void *'>
	<var: 'baPtr' type: #'void *'>

	start := self stackIntegerValue: 0.
	(self failed or: [ start < 1 ]) ifTrue:
			[^self primitiveFailForOSError: 1 ].

	oop := self stackObjectValue: 1.
	(self failed
		or: [(self is: oop KindOf: 'ByteArray') not]) ifTrue:
			[^self primitiveFailForOSError: 2 ].
	baPtr := self arrayValueOf: oop.
	baSize := self stSizeOf: oop.

	size := self stackIntegerValue: 2.
	(self failed or: [ size < 0 ]) ifTrue:
		[^self primitiveFailForOSError: 3 ].

	(start + size - 1 > baSize) ifTrue:
		[ ^self primitiveFailForOSError: 4 ].

	"Currently assumes this is a 64 bit platform"
	oop := self stackObjectValue: 3.
	(self failed
		or: [(self is: oop KindOf: 'ExternalAddress') not]) ifTrue:
			[^self primitiveFailForOSError: 5 ].
	externalPtr := self arrayValueOf: oop.
	externalAddress := self cCode: '*externalPtr'.
	externalAddress = 0 ifTrue:
			[^self primitiveFailForOSError: 6 ].

	baPtr := baPtr + start - 1.

	self cCode: 'memcpy(baPtr, externalAddress, size)'.
	self methodReturnInteger: size.
]
