{
    InterpreterPrimitives >> #primitiveFail.
    InterpreterPrimitives >> #primitiveFailFor:.


    StackInterpreter >> #methodReturnValue:.
    StackInterpreter >> #methodReturnBool:.
    StackInterpreter >> #methodReturnFloat:.
    StackInterpreter >> #methodReturnInteger:.
    StackInterpreter >> #methodReturnReceiver.
    StackInterpreter >> #methodArgumentCount.

    StackInterpreter >> #stackValue:.
    StackInterpreter >> #stackObjectValue:.
    StackInterpreter >> #stackIntegerValue:.
    StackInterpreter >> #stackFloatValue:.

    StackInterpreter >> #stSizeOf:.
    StackInterpreter >> #stObject:at:.
    StackInterpreter >> #stObject:at:put:.

    StackInterpreter >> #addressCouldBeClassObj:.
    StackInterpreter >> #is:KindOfClass:.

    CoInterpreter >> #instVar:ofContext:.

    SpurMemoryManager >> #falseObject.
    SpurMemoryManager >> #trueObject.
    SpurMemoryManager >> #nilObject.
    SpurMemoryManager >> #classArray.
    SpurMemoryManager >> #classExternalAddress.
    SpurMemoryManager >> #classString.
    SpurMemoryManager >> #isOld:.
    SpurMemoryManager >> #isYoung:.
    SpurMemoryManager >> #possibleOldObjectStoreInto:.
    SpurMemoryManager >> #possiblePermObjectStoreInto:value:.
    SpurMemoryManager >> #fetchClassOfNonImm:.

    SpurMemoryManager >> #instantiateClass:isPinned:.
    SpurMemoryManager >> #instantiateClass:indexableSize:.
    SpurMemoryManager >> #instantiateClass:indexableSize:isPinned:.
    SpurMemoryManager >> #firstFixedField:.
    SpurMemoryManager >> #firstIndexableField:.
    SpurMemoryManager >> #newHashBitsOf:.
    SpurMemoryManager >> #hashBitsOf:.
    SpurMemoryManager >> #ensureBehaviorHash:.
    SpurMemoryManager >> #fetchPointer:ofObject:.
    SpurMemoryManager >> #firstBytePointerOfDataObject:.
    SpurMemoryManager >> #isOopForwarded:.

    Spur32BitMemoryManager >> #integerObjectOf:.
    Spur32BitMemoryManager >> #floatObjectOf:.
    Spur32BitMemoryManager >> #floatValueOf:.
    Spur32BitMemoryManager >> #isFloatInstance:.
    
    Spur64BitMemoryManager >> #integerObjectOf:.
    Spur64BitMemoryManager >> #floatObjectOf:.
    Spur64BitMemoryManager >> #floatValueOf:.
    Spur64BitMemoryManager >> #isFloatInstance:.
}
do: [ :eachMethod |
    (RBAddPragmaTransformation pragma: '<export: true>' inMethod: eachMethod selector inClass: eachMethod methodClass name)
        asRefactoring execute ].


SpurMemoryManager compile: 'getStatFullGCUsecs
    <api>
    ^ statFullGCUsecs'.

SpurMemoryManager compile: 'getStatScavengeGCUsecs
    <api>
    ^ statScavengeGCUsecs'.

SpurMemoryManager compile: 'getOldSpaceMemoryStart
    <api>
    <export: true>
    ^ self firstObject'.

SpurMemoryManager compile: 'getOldSpaceMemoryEnd
    <api>
    <export: true>
    ^ memoryMap oldSpaceEnd'.
    
SpurMemoryManager compile: 'getEdenSpaceMemoryStart
    <api>
    <export: true>
    ^ scavenger eden start'.

SpurMemoryManager compile: 'getEdenSpaceMemoryEnd
    <api>
    <export: true>
    ^ freeStart'.
    
SpurMemoryManager compile: 'getPastSpaceMemoryStart
    <api>
    <export: true>
    ^ scavenger pastSpace start'.

SpurMemoryManager compile: 'getPastSpaceMemoryEnd
    <api>
    <export: true>
    ^ pastSpaceStart'.
   
SpurMemoryManager compile: 'getObjectAfter: objOop limit: limit
    <api>
    <export: true>
    ^ self objectAfter: objOop limit: limit'.

StackInterpreter compile: 'getThisContext
    <export: true>
    ^ self ensureFrameIsMarried: framePointer SP: stackPointer'.

InterpreterPrimitives compile: 'createNewMethod: class header: header bytecodeCount: bytecodeCount
    <api>
    <export: true>
    <var: ''bytecodeCount'' type: ''sqInt''>

	| size theMethod literalCount |

	literalCount := objectMemory literalCountOfMethodHeader: header.
	size := literalCount + LiteralStart * objectMemory bytesPerOop
	        + bytecodeCount.
	theMethod := objectMemory
		             instantiateCompiledMethodClass: class
		             indexableSize: size.
	theMethod ifNil: [ ^ theMethod ].
	objectMemory
		storePointerUnchecked: HeaderIndex
		ofObject: theMethod
		withValue: header.
	1 to: literalCount do: [ :i |
		objectMemory
			storePointer: i
			ofObject: theMethod
			withValue: objectMemory nilObject ].
	^ theMethod'.

StackInterpreter compile: 'stContextSize: aContext
    | sz |
    <export: true>

    self writeBackHeadFramePointers.
    sz := self stackPointerForMaybeMarriedContext: aContext.
    ^ sz'.


CogAbstractInstruction compile: 'unalignedLongAt: byteAddress put: aWord
    <cmacro: ''(byteAddress,aWord) memcpy((char*)byteAddress, &aWord, sizeof(sqInt))''>

    ^ objectMemory unalignedLongAt: byteAddress put: aWord'.

CogX64Compiler compile: 'unalignedLong32At: byteAddress put: aWord
	<cmacro: ''(inst,byteAddress,aWord) memcpy((char*)byteAddress, &aWord, sizeof(int32_t))''>

	objectMemory
		byteAt: byteAddress + 0 put:  (aWord bitAnd: 16rFF);
		byteAt: byteAddress + 1 put: ((aWord >> 8) bitAnd: 16rFF);
		byteAt: byteAddress + 2 put: ((aWord >> 16) bitAnd: 16rFF);
		byteAt: byteAddress + 3 put: ((aWord >> 24) bitAnd: 16rFF).
	^aWord'.

Smalltalk snapshot: true andQuit: true