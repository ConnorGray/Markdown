BeginPackage["ConnorGray`Markdown`"]

MarkdownElement

MarkdownParse

Begin["`Private`"]

(* Install any missing dependencies. *)
PacletInstall /@ PacletObject["ConnorGray/NotebookWebsiteTools"]["Dependencies"]

Needs["Wolfram`ErrorTools`"]

(*========================================================*)
(* Eagerly load the underlying implementation library     *)
(*========================================================*)

$functions = LibraryFunctionLoad[
	"libwolfram_markdown_link",
	"load_wolfram_markdown_link",
	LinkObject,
	LinkObject
][]

RaiseAssert[
	MatchQ[$functions, _?AssociationQ],
	"Error loading Markdown implementation library: loaded functions has unexpected form: ``",
	InputForm[$functions]
];

(*========================================================*)

SetFallthroughError[MarkdownParse]

MarkdownParse[s_?StringQ] := Module[{result},
	result = $functions["parse_markdown"][s];

	result
]

End[]

EndPackage[]