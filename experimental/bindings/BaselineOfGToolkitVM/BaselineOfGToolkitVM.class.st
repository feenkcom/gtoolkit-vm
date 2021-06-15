Class {
	#name : #BaselineOfGToolkitVM,
	#superclass : #BaselineOf,
	#category : #BaselineOfGToolkitVM
}

{ #category : #baselines }
BaselineOfGToolkitVM >> baseline: spec [
	<baseline>
	spec for: #common do: [
		spec
			baseline: 'ThreadedFFI'
			with: [ spec repository: 'github://pharo-project/threadedFFI-Plugin' ].

		spec
			package: 'GToolkit-VM'
			with: [ spec requires: #(ThreadedFFI) ] ]
]
