Extension { #name : #SpurMemoryManager }

{ #category : #'*GToolkit-VMMaker-AddOns' }
SpurMemoryManager >> pinObject: objOop [
	"Attempt to pin objOop, which must not be immediate.
	 If the attempt succeeds answer objOop's (possibly moved) oop.
	 If the attempt fails, which can only occur if there is no memory, answer 0.
	 GlamorousToolkit patch."
	<inline: false>
	| oldClone seg |
	<var: #seg type: #'SpurSegmentInfo *'>
	self assert: (self isNonImmediate: objOop).
	"Previous versions of this method would attempt to keep pinned objects in the same old space segment on the following assumptions:
	- Pinning is rare
	- Fragmentation is bad
	- Pinned objects will stay pinned (unstated assumption)
	For GT, pinning is not rare (GT graphics uses ffi, which requires pinning) and objects do not stay pinned (we unpin objects ASAP after ffi operations).
	So, simply mark old objects as already pinned and move new objects to old space using the regular algorithm (non-grouping)"
	(memoryMap isOldObject: objOop) ifTrue:
		 [seg := segmentManager segmentContainingObj: objOop.
		 self setIsPinnedOf: objOop to: true.
		 seg containsPinned: true.
		 ^objOop].
	oldClone := self cloneInOldSpace: objOop forPinning: false.
	oldClone ~= 0 ifTrue:
		[
		seg := segmentManager segmentContainingObj: oldClone.
		seg containsPinned: true.
		becomeEffectsFlags := self becomeEffectFlagsFor: objOop.
		self setIsPinnedOf: oldClone to: true.
		self forward: objOop to: oldClone.
		self followSpecialObjectsOop.
		coInterpreter postBecomeAction: becomeEffectsFlags.
		self postBecomeScanClassTable: becomeEffectsFlags.
		becomeEffectsFlags := 0].
	^oldClone
]
