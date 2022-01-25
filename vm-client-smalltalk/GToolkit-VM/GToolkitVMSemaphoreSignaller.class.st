"
! VM Semaphore signaller
That should be used to signal a semaphore from a OS thread to let image know that it should process something
"
Class {
	#name : #GToolkitVMSemaphoreSignaller,
	#superclass : #FFIExternalObject,
	#instVars : [
		'thunk',
		'semaphore'
	],
	#category : #'GToolkit-VM'
}

{ #category : #printing }
GToolkitVMSemaphoreSignaller >> printOn: aStream [

	"Append a sequence of characters to aStream that identify the receiver."

	super printOn: aStream.
	aStream
		nextPutAll: ' handle: ';
		print: handle.
	aStream
		nextPutAll: ' thunk: ';
		print: thunk
]

{ #category : #accessing }
GToolkitVMSemaphoreSignaller >> semaphore [

	^ semaphore
]

{ #category : #accessing }
GToolkitVMSemaphoreSignaller >> semaphore: anObject [

	semaphore := anObject
]

{ #category : #accessing }
GToolkitVMSemaphoreSignaller >> thunk [

	^ thunk
]

{ #category : #accessing }
GToolkitVMSemaphoreSignaller >> thunk: anObject [

	thunk := anObject
]
