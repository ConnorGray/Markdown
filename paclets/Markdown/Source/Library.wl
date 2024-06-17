BeginPackage["ConnorGray`Markdown`Library`"]

$LibraryFunctions

Begin["`Private`"]

Needs["Wolfram`ErrorTools`"]

(*========================================================*)
(* Eagerly load the underlying implementation library     *)
(*========================================================*)

$LibraryFunctions = LibraryFunctionLoad[
	"libwolfram_markdown_link",
	"load_wolfram_markdown_link",
	LinkObject,
	LinkObject
][]

RaiseAssert[
	MatchQ[$LibraryFunctions, _?AssociationQ],
	"Error loading Markdown implementation library: loaded functions has unexpected form: ``",
	InputForm[$LibraryFunctions]
];


End[]

EndPackage[]