BeginPackage["ConnorGray`Markdown`ToMarkdownString`"]

Begin["`Private`"]

Needs["Wolfram`ErrorTools`"]

Needs["ConnorGray`Markdown`"]
Needs["ConnorGray`Markdown`Library`"]

(*========================================================*)

SetFallthroughError[ToMarkdownString]

ToMarkdownString[
	markdown0 : _MarkdownElement | {___MarkdownElement},
	indent : _?IntegerQ : 0
] := WrapRaised[
	MarkdownError,
	"Error in ToMarkdownString for ``",
	InputForm[markdown0]
] @ Module[{
	markdown = Replace[markdown0, m:Except[_?ListQ] :> {m}]
},
	$LibraryFunctions["markdown_ast_to_markdown"][markdown]
]

(*========================================================*)

End[]

EndPackage[]