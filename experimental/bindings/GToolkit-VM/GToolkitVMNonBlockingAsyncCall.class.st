Class {
	#name : #GToolkitVMNonBlockingAsyncCall,
	#superclass : #TFExternalAsyncCall,
	#category : #'GToolkit-VM'
}

{ #category : #'private - semaphore' }
GToolkitVMNonBlockingAsyncCall >> deregisterSemaphore [
]

{ #category : #operations }
GToolkitVMNonBlockingAsyncCall >> doExecuteOn: aRunner [
	
	| aTaskAddress | 
	
	aTaskAddress := aRunner
		executeFunction: function
		withArguments: parameterArray
		usingSemaphore: self semaphoreIndex.
	
	"we do not wait on the semaphore since we do not expect a return value"

	^ nil
]

{ #category : #operations }
GToolkitVMNonBlockingAsyncCall >> executeOn: aRunner [
	"Performs a callout using the asynchronous threaded FFI plugin.
	 This method schedules the execution of a function in the threaded FFI plugin and 
	 then waits for its result in a semaphore.
	 When the semaphore is signaled, the result is available in an external value holder
	 from where the value is read.
	 This method contains some boilerplate code to allocate and release external 
	 value holders for the arguments and the return value."
	
	^ [
			semaphore := aRunner semaphorePool takeOne.
			self prepareExecution.
			self doExecuteOn: aRunner ]
		ensure: [
			self cleanUp.
			semaphore reset.
			aRunner semaphorePool returnOne: semaphore ]
]

{ #category : #initialization }
GToolkitVMNonBlockingAsyncCall >> initialize [

]

{ #category : #'private - semaphore' }
GToolkitVMNonBlockingAsyncCall >> registerSemaphore [
]

{ #category : #accessing }
GToolkitVMNonBlockingAsyncCall >> reset [

	self semaphore signalAll
]

{ #category : #accessing }
GToolkitVMNonBlockingAsyncCall >> semaphore [

	^ semaphore semaphore
]

{ #category : #accessing }
GToolkitVMNonBlockingAsyncCall >> semaphoreIndex [

	^ semaphore index
]
