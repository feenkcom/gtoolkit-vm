Class {
	#name : #GtPrimitiveExamples,
	#superclass : #Object,
	#traits : 'TAssertable',
	#classTraits : 'TAssertable classTrait',
	#category : #'GToolkit-VMMaker-AddOns-Examples'
}

{ #category : #primitives }
GtPrimitiveExamples >> primitiveCopyFrom: src at: srcStart to: dest at: destStart size: size [
	<primitive: 'primitiveCopyFromAtToAtSize'>
	^ self primitiveFailed
]

{ #category : #tests }
GtPrimitiveExamples >> testByteArrayToExternalAddressBadArguments [
	<gtExample>
	| ea ba null |

	ea := ExternalAddress allocate: 8.
	ba := ByteArray new: 8.
	"Source isn't a ByteArray"
	self should:
		[ self
			primitiveCopyFrom: Object new
			at: 1
			to: ea
			at: 1
			size: 8 ]
		raise: PrimitiveFailed.

	"Null address isn't allowed"
	null := ByteArray new: 8.
	ExternalAddress adoptInstance: null.
	self should:
		[ self
			primitiveCopyFrom: ba
			at: 1
			to: null
			at: 1
			size: 8 ]
		raise: PrimitiveFailed.

	"Destination isn't a ExternalAddress"
	self should:
		[ self
			primitiveCopyFrom: ba
			at: 1
			to: Object new
			at: 1
			size: 8 ]
		raise: PrimitiveFailed.

	"Size isn't an integer"
	self should:
		[ self
			primitiveCopyFrom: ba
			at: 1
			to: ea
			at: 1
			size: 8.3 ]
		raise: PrimitiveFailed.

	"Size must be >= 0"
	self should:
		[ self
			primitiveCopyFrom: ba
			at: 1
			to: ea
			at: 1
			size: -1 ]
		raise: PrimitiveFailed.

	"start isn't an integer"
	self should:
		[ self
			primitiveCopyFrom: ba
			at: 1.2
			to: ea
			at: 1
			size: 8 ]
		raise: PrimitiveFailed.

	"start must be >= 1"
	self should:
		[ self
			primitiveCopyFrom: ba
			at: 0
			to: ea
			at: 1
			size: 8 ]
		raise: PrimitiveFailed.

	"ByteArray buffer overflow from starting position"
	self should:
		[ self
			primitiveCopyFrom: ba
			at: 10
			to: ea
			at: 1
			size: 8 ]
		raise: PrimitiveFailed.

	"ByteArray buffer overflow from size"
	self should:
		[ self
			primitiveCopyFrom: ba
			at: 1
			to: ea
			at: 1
			size: 80  ]
		raise: PrimitiveFailed.

	ea free.
	^ ba
]

{ #category : #tests }
GtPrimitiveExamples >> testByteArrayToExternalStartingAt1 [
	<gtExample>
	| ea ba size result |
	size := 24.
	ea := ExternalAddress allocate: size.
	1 to: size do: [ :i | ea byteAt: i put: i \\ 256 ].
	ba := ByteArray new: size.
	result := self
			primitiveCopyFrom: ba
			at: 1
			to: ea
			at: 1
			size: size.
	self assert: result equals: size.
	self assert: (ea copyFrom: 1 to: size) equals: ba.
	ea free.
	^ ba
]

{ #category : #tests }
GtPrimitiveExamples >> testByteArrayToExternalStartingAt11 [
	<gtExample>
	| ea ba size result |
	size := 24.
	ea := ExternalAddress allocate: size+10.
	ba := ByteArray new: size.
	1 to: size do: [ :i | ba byteAt: i put: i \\ 256 ].
	result := self
			primitiveCopyFrom: ba
			at: 1
			to: ea
			at: 11
			size: size.
	self assert: result equals: size.
	self assert: (ea copyFrom: 11 to: size+10) equals: ba.
	ea free.
	^ ba
]

{ #category : #tests }
GtPrimitiveExamples >> testExternalAddressToByteArrayBadArguments [
	<gtExample>
	| ea ba null |

	ea := ExternalAddress allocate: 8.
	ba := ByteArray new: 8.
	"Source isn't an ExternalAddress"
	self should:
		[ self
			primitiveCopyFrom: Object new
			at: 1
			to: ba 
			at: 1
			size: 8 ]
		raise: PrimitiveFailed.

	"Null address isn't allowed"
	null := ByteArray new: 8.
	ExternalAddress adoptInstance: null.
	self should:
		[ self
			primitiveCopyFrom: null
			at: 1
			to: ba 
			at: 1
			size: 8 ]
		raise: PrimitiveFailed.

	"Destination isn't a ByteArray"
	self should:
		[ self
			primitiveCopyFrom: ea
			at: 1
			to: Object new
			at: 1
			size: 8 ]
		raise: PrimitiveFailed.

	"Size isn't an integer"
	self should:
		[ self
			primitiveCopyFrom: ea
			at: 1
			to: ba
			at: 1
			size: 8.3 ]
		raise: PrimitiveFailed.

	"Size must be >= 0"
	self should:
		[ self
			primitiveCopyFrom: ea
			at: 1
			to: ba
			at: 1
			size: -1 ]
		raise: PrimitiveFailed.

	"start isn't an integer"
	self should:
		[ self
			primitiveCopyFrom: ea
			at: 1
			to: ba
			at: 1.2
			size: 8 ]
		raise: PrimitiveFailed.

	"start must be >= 1"
	self should:
		[ self
			primitiveCopyFrom: ea
			at: 1
			to: ba
			at: 0
			size: 8 ]
		raise: PrimitiveFailed.

	"ByteArray buffer overflow from starting position"
	self should:
		[ self
			primitiveCopyFrom: ea
			at: 1
			to: ba 
			at: 10
			size: 8 ]
		raise: PrimitiveFailed.

	"ByteArray buffer overflow from size"
	self should:
		[ self
			primitiveCopyFrom: ea
			at: 1
			to: ba 
			at: 1
			size: 80 ]
		raise: PrimitiveFailed.

	ea free.
	^ ba
]

{ #category : #tests }
GtPrimitiveExamples >> testExternalToByteArrayStartingAt1 [
	<gtExample>
	| ea ba size result |
	size := 24.
	ea := ExternalAddress allocate: size.
	1 to: size do: [ :i | ea byteAt: i put: i \\ 256 ].
	ba := ByteArray new: size.
	result := self
			primitiveCopyFrom: ea
			at: 1
			to: ba
			at: 1
			size: size.
	self assert: result equals: size.
	self assert: ba equals: (ea copyFrom: 1 to: size).
	ea free.
	^ ba
]

{ #category : #tests }
GtPrimitiveExamples >> testExternalToByteArrayStartingAt11 [
	<gtExample>
	| ea ba size result |
	size := 24.
	ea := ExternalAddress allocate: size.
	1 to: size do: [ :i | ea byteAt: i put: i \\ 256 ].
	ba := ByteArray new: size + 10.
	result := self
			primitiveCopyFrom: ea
			at: 1
			to: ba
			at: 11
			size: size.
	self assert: result equals: size.
	self assert: (ba copyFrom: 11 to: ba size) equals: (ea copyFrom: 1 to: size).
	ea free.
	^ ba
]
