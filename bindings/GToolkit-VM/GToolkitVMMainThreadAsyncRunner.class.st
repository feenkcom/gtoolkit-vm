"
!Main thread runner
Allows users to perform an FFI callout on the main thread of ${class:GToolkitVM}$
"
Class {
	#name : #GToolkitVMMainThreadAsyncRunner,
	#superclass : #GToolkitVMMainThreadRunner,
	#category : #'GToolkit-VM'
}

{ #category : #'private - ffi' }
GToolkitVMMainThreadAsyncRunner >> primitiveExecuteFunction: aTFExternalFunction withArguments: arguments usingSemaphore: anInteger [
	<primitive: 'primitiveMainThreadCalloutGToolkitVM'>
	
	self primitiveFailed
]

{ #category : #executing }
GToolkitVMMainThreadAsyncRunner >> readReturnValueFromTask: anExternalAddress [
	^ self primitiveReadReturnValueFromTask: anExternalAddress
]
