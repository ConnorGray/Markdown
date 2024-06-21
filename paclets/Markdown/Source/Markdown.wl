BeginPackage["ConnorGray`Markdown`"]

(*------------------------------------*)
(* Symbolic Markdown                  *)
(*------------------------------------*)

MarkdownElement

(*------------------------------------*)
(* Operations on Markdown             *)
(*------------------------------------*)

MarkdownParse
ToMarkdownString

(*------------------------------------*)
(* Markdown <=> Notebook Conversion   *)
(*------------------------------------*)

CreateMarkdownNotebook
ConvertToMarkdownElement

MarkdownError

ExportMarkdown::usage = "ExportMarkdown[dest$, expr$] exports data to Markdown."
ExportMarkdownString::usage = "ExportMarkdownString[expr$] exports data to a Markdown string."

Begin["`Private`"]

(* Install any missing dependencies. *)
PacletInstall /@ PacletObject["ConnorGray/NotebookWebsiteTools"]["Dependencies"]

Needs["Wolfram`ErrorTools`"]

Needs["ConnorGray`Markdown`Library`"]
Needs["ConnorGray`Markdown`ConvertToMarkdown`"]
Needs["ConnorGray`Markdown`ToMarkdownString`"]

CreateErrorType[MarkdownError, {}]


(*========================================================*)

SetFallthroughError[MarkdownParse]

MarkdownParse[s_?StringQ] := Module[{result},
	result = $LibraryFunctions["parse_markdown"][s];

	result
]

(*========================================================*)

SetFallthroughError[ExportMarkdown]

ExportMarkdown[dest_, expr_] := Module[{
	markdown
},
	markdown = ExportMarkdownString[expr];

	RaiseAssert[
		MatchQ[markdown, {___MarkdownElement}],
		"Converstion to markdown has unexpected result: ``", markdown
	];

	Export[dest, markdown, "Text"]
]

(*========================================================*)

SetFallthroughError[ExportMarkdownString]

ExportMarkdownString[expr_] := Handle[_Failure] @ Module[{
	markdown
},
	markdown = RaiseConfirm @ ConvertToMarkdownElement[expr];

	markdown = RaiseConfirm @ ToMarkdownString[markdown];

	RaiseAssert[
		StringQ[markdown],
		"Expected Markdown string result, got: ``", InputForm[markdown]
	];

	markdown
]

End[]

EndPackage[]