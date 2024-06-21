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

ConvertToMarkdownElement

MarkdownError

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

End[]

EndPackage[]