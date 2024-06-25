Class {
	#name : #GtPrimitiveExamples,
	#superclass : #Object,
	#traits : 'TAssertable',
	#classTraits : 'TAssertable classTrait',
	#category : #'GToolkit-VMMaker-AddOns-Examples'
}

{ #category : #tests }
GtPrimitiveExamples >> testByteArrayToExternalAddressBadArguments [
	<gtExample>
	| ea ba null |

	ea := ExternalAddress allocate: 8.
	ba := ByteArray new: 8.
	"Source isn't a ByteArray"
	self should:
		[ ea
			primitiveCopy: Object new
			startingAt: 1
			size: 8
			toExternal: ea 
			 ]
		raise: PrimitiveFailed.

	"Cheating with a ByteArray isn't allowed"
	self should:
		[ ea
			primitiveCopy: ba
			startingAt: 1
			size: 8
			toExternal: (ByteArray new: 8) ]
		raise: PrimitiveFailed.

	"Null address isn't allowed"
	null := ByteArray new: 8.
	ExternalAddress adoptInstance: null.
	self should:
		[ ea
			primitiveCopy: ba
			startingAt: 1
			size: 8
			toExternal: null ]
		raise: PrimitiveFailed.

	"Destination isn't a ExternalAddress"
	self should:
		[ ea
			primitiveCopy: ba
			startingAt: 1
			size: 8
			toExternal: Object new ]
		raise: PrimitiveFailed.

	"Size isn't an integer"
	self should:
		[ ea
			primitiveCopy: ba
			startingAt: 1
			size: 8.3
			toExternal: ea ]
		raise: PrimitiveFailed.

	"Size must be >= 0"
	self should:
		[ ea
			primitiveCopy: ba
			startingAt: 1
			size: -1
			toExternal: ea ]
		raise: PrimitiveFailed.

	"start isn't an integer"
	self should:
		[ ea
			primitiveCopy: ba
			startingAt: 1.2
			size: 8
			toExternal: ea ]
		raise: PrimitiveFailed.

	"start must be >= 1"
	self should:
		[ ea
			primitiveCopy: ba
			startingAt: 0
			size: 8
			toExternal: ea ]
		raise: PrimitiveFailed.

	"ByteArray buffer overflow from starting position"
	self should:
		[ ea
			primitiveCopy: ba
			startingAt: 10
			size: 8
			toExternal: ea ]
		raise: PrimitiveFailed.

	"ByteArray buffer overflow from size"
	self should:
		[ ea
			primitiveCopy: ba
			startingAt: 1
			size: 80
			toExternal: ea  ]
		raise: PrimitiveFailed.

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
		[ ea
			primitiveCopyExternal: Object new
			size: 8
			to: ba 
			startingAt: 1 ]
		raise: PrimitiveFailed.

	"Cheating with a ByteArray isn't allowed"
	self should:
		[ ea
			primitiveCopyExternal: (ByteArray new: 8)
			size: 8
			to: ba 
			startingAt: 1 ]
		raise: PrimitiveFailed.

	"Null address isn't allowed"
	null := ByteArray new: 8.
	ExternalAddress adoptInstance: null.
	self should:
		[ ea
			primitiveCopyExternal: null
			size: 8
			to: ba 
			startingAt: 1 ]
		raise: PrimitiveFailed.

	"Destination isn't a ByteArray"
	self should:
		[ ea
			primitiveCopyExternal: ea
			size: 8
			to: Object new
			startingAt: 1 ]
		raise: PrimitiveFailed.

	"Size isn't an integer"
	self should:
		[ ea
			primitiveCopyExternal: ea
			size: 8.3
			to: ba
			startingAt: 1 ]
		raise: PrimitiveFailed.

	"Size must be >= 0"
	self should:
		[ ea
			primitiveCopyExternal: ea
			size: -1
			to: ba
			startingAt: 1 ]
		raise: PrimitiveFailed.

	"start isn't an integer"
	self should:
		[ ea
			primitiveCopyExternal: ea
			size: 8
			to: ba
			startingAt: 1.2 ]
		raise: PrimitiveFailed.

	"start must be >= 1"
	self should:
		[ ea
			primitiveCopyExternal: ea
			size: 8
			to: ba
			startingAt: 0 ]
		raise: PrimitiveFailed.

	"ByteArray buffer overflow from starting position"
	self should:
		[ ea
			primitiveCopyExternal: ea
			size: 8
			to: ba 
			startingAt: 10 ]
		raise: PrimitiveFailed.

	"ByteArray buffer overflow from size"
	self should:
		[ ea
			primitiveCopyExternal: ea
			size: 80
			to: ba 
			startingAt: 1 ]
		raise: PrimitiveFailed.

	ea free.
	^ ba
]

{ #category : #tests }
GtPrimitiveExamples >> testExternalToObjectStartingAt1 [
	<gtExample>
	| ea ba size result |
	size := 24.
	ea := ExternalAddress allocate: size.
	1 to: size do: [ :i | ea byteAt: i put: i \\ 256 ].
	ba := ByteArray new: size.
	result := ea
			primitiveCopyExternal: ea
			size: size
			to: ba
			startingAt: 1.
	self assert: result equals: size.
	self assert: ba equals: (ea copyFrom: 1 to: size).
	ea free.
	^ ba
]

{ #category : #tests }
GtPrimitiveExamples >> testExternalToObjectStartingAt10 [
	<gtExample>
	| ea ba size result |
	size := 24.
	ea := ExternalAddress allocate: size.
	1 to: size do: [ :i | ea byteAt: i put: i \\ 256 ].
	ba := ByteArray new: size + 10.
	result := ea
			primitiveCopyExternal: ea
			size: size
			to: ba
			startingAt: 11.
	self assert: result equals: size.
	self assert: (ba copyFrom: 11 to: ba size) equals: (ea copyFrom: 1 to: size).
	ea free.
	^ ba
]

{ #category : #tests }
GtPrimitiveExamples >> testObjectToExternalStartingAt1 [
	<gtExample>
	| ea ba size result |
	size := 24.
	ea := ExternalAddress allocate: size.
	1 to: size do: [ :i | ea byteAt: i put: i \\ 256 ].
	ba := ByteArray new: size.
	result := ea
			primitiveCopy: ba
			startingAt: 1
			size: size
			toExternal: ea.
	self assert: result equals: size.
	self assert: (ea copyFrom: 1 to: size) equals: ba.
	ea free.
	^ ba
]
