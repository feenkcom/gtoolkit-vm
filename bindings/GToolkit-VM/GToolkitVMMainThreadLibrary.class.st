"
! Main thread library
Use me as `FFILibrary` if you want to query ${class:GToolkitVM}$ on a main thread
"
Class {
	#name : #GToolkitVMMainThreadLibrary,
	#superclass : #GToolkitVMLibrary,
	#category : #'GToolkit-VM'
}

{ #category : #accessing }
GToolkitVMMainThreadLibrary >> runner [
	^ GToolkitVM current isOnWorkerThread
		ifTrue: [ GToolkitVMMainThreadRunner uniqueInstance ]
		ifFalse: [ TFSameThreadRunner uniqueInstance ]
]
