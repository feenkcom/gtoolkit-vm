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



    SpurMemoryManager >> #falseObject.
    SpurMemoryManager >> #trueObject.
    SpurMemoryManager >> #nilObject.

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
    Spur64BitMemoryManager >> #integerObjectOf:.

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

Smalltalk snapshot: true andQuit: true