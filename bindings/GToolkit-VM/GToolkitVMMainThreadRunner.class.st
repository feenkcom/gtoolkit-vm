"
!Main thread runner
Allows users to perform an FFI callout on the main thread of ${class:GToolkitVM}$
"
Class {
	#name : #GToolkitVMMainThreadRunner,
	#superclass : #TFRunner,
	#classInstVars : [
		'uniqueInstance'
	],
	#category : #'GToolkit-VM'
}

{ #category : #accessing }
GToolkitVMMainThreadRunner class >> ffiLibrary [
	^ GToolkitVMLibrary
]

{ #category : #accessing }
GToolkitVMMainThreadRunner class >> uniqueInstance [
	^ uniqueInstance ifNil: [ uniqueInstance := self basicNew initialize ]
]

{ #category : #executing }
GToolkitVMMainThreadRunner >> doInitialize [
	super doInitialize.
	
	"we do not use this handle, however, if we do not initialize it then the isNull check fails"
	handle := ExternalAddress newPointer
]

{ #category : #executing }
GToolkitVMMainThreadRunner >> executeFunction: aTFExternalFunction withArguments: arguments usingSemaphore: anInteger [
	^ self
		primitiveExecuteFunction: aTFExternalFunction
		withArguments: arguments
		usingSemaphore: anInteger
]

{ #category : #resources }
GToolkitVMMainThreadRunner >> ffiLibrary [
	^ self class ffiLibrary
]

{ #category : #'private - deprecated' }
GToolkitVMMainThreadRunner >> primCall: aGToolkitVM cif: aCifHandle function: aFunctionHandle arguments: anArgumentsHandle result: aResultHandle semaphore: aSemaphoreIndex [
	^ self ffiCall: #(void gtoolkit_vm_main_thread_callout(
		void* aGToolkitVM,
		void* aCifHandle,
		void* aFunctionHandle,
		void* anArgumentsHandle,
		void* aResultHandle,
		long aSemaphoreIndex ))
]

{ #category : #'private - ffi' }
GToolkitVMMainThreadRunner >> primitiveExecuteFunction: aTFExternalFunction withArguments: arguments usingSemaphore: anInteger [
	<primitive: 'primitiveMainThreadCalloutGToolkitVM'>
	
	self primitiveFailed
]

{ #category : #'private - deprecated' }
GToolkitVMMainThreadRunner >> primitivePerformWorkerCall: aTFExternalFunction withArguments: argumentHolder withReturnHolder: aReturnHolder usingSemaphore: anInteger [
	"Obsolete in Pharo9"

	self
		primCall: GToolkitVM current
		cif: aTFExternalFunction definition getHandle
		function: aTFExternalFunction address
		arguments: argumentHolder
		result: aReturnHolder
		semaphore: anInteger
]

{ #category : #'private - ffi' }
GToolkitVMMainThreadRunner >> primitiveReadReturnValueFromTask: anExternalAddress [
	<primitive: 'primitiveExtractReturnValueGToolkitVM'>

	self primitiveFailed
]

{ #category : #executing }
GToolkitVMMainThreadRunner >> readReturnValueFromTask: anExternalAddress [
	^ self primitiveReadReturnValueFromTask: anExternalAddress
]
