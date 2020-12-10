"
!GToolkit VM
Provides access to the GToolkit VM and implements query api.
"
Class {
	#name : #GToolkitVM,
	#superclass : #FFIExternalObject,
	#classInstVars : [
		'current'
	],
	#category : #'GToolkit-VM'
}

{ #category : #accessing }
GToolkitVM class >> current [
	(current isNil or: [ current isNull ])
		ifTrue: [ current := self new setHandle: (self primitiveGetAddressOfGToolkitVM pinInMemory; yourself) ].
	^ current
]

{ #category : #resources }
GToolkitVM class >> ffiLibrary [
	^ GToolkitVMSameThreadLibrary
]

{ #category : #'private - primitive' }
GToolkitVM class >> primitiveGetAddressOfGToolkitVM [
	<primitive: #primitiveGetAddressOfGToolkitVM>

	^ ExternalAddress null
]

{ #category : #resources }
GToolkitVM >> ffiLibrary [
	^ self class ffiLibrary
]

{ #category : #testing }
GToolkitVM >> isOnWorkerThread [
	<return: #Boolean>

	^ [ self primIsOnWorkerThread ] on: Error do: [ false ]
]

{ #category : #testing }
GToolkitVM >> isRunningGToolkitVM [
	<return: #Boolean>

	^ self isNull not
]

{ #category : #accessing }
GToolkitVM >> majorVersion [
	^ self primMajorVersion
]

{ #category : #accessing }
GToolkitVM >> minorVersion [
	^ self primMinorVersion
]

{ #category : #'private - ffi' }
GToolkitVM >> primIsOnWorkerThread [
	^ self ffiCall: #(bool gtoolkit_vm_is_on_worker_thread(self))
]

{ #category : #'private - ffi' }
GToolkitVM >> primMajorVersion [
	^ self ffiCall: #(size_t gtoolkit_vm_major_version(self))
]

{ #category : #'private - ffi' }
GToolkitVM >> primMinorVersion [
	^ self ffiCall: #(size_t gtoolkit_vm_minor_version(self))
]

{ #category : #'private - ffi' }
GToolkitVM >> primWakeUp [
	^ self ffiCall: #(void gtoolkit_vm_wake_up(self))
]

{ #category : #accessing }
GToolkitVM >> version [
	^ { self majorVersion . self minorVersion }
]

{ #category : #'vm - actions' }
GToolkitVM >> wakeUp [
	"Wake up the main thread"

	self primWakeUp
]
