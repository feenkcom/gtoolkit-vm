Class {
	#name : #GToolkitVMMainThreadNonBlockingRunner,
	#superclass : #GToolkitVMMainThreadRunner,
	#category : #'GToolkit-VM'
}

{ #category : #executing }
GToolkitVMMainThreadNonBlockingRunner >> invokeFunction: aTFExternalFunction withArguments: aCollection [
	^ (GToolkitVMNonBlockingAsyncCall forFunction: aTFExternalFunction)
		parameters: aCollection;
		executeOn: self
]

{ #category : #'private - ffi' }
GToolkitVMMainThreadNonBlockingRunner >> primitiveExecuteFunction: aTFExternalFunction withArguments: arguments usingSemaphore: anInteger [
	<primitive: 'primitiveMainThreadCalloutGToolkitVM'>
	
	self primitiveFailed
]

{ #category : #executing }
GToolkitVMMainThreadNonBlockingRunner >> readReturnValueFromTask: anExternalAddress [
	^ nil
]
