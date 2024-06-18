BeginPackage["ConnorGray`Markdown`"]

MarkdownElement

MarkdownParse

ToMarkdownString

ConvertToMarkdown

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