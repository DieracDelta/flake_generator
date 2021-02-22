module FzfLib (fzfQuery, isFlake) where


import Data.List as L
import Turtle as T
import Data.Text as Te
import Data.Monoid as M

fzfQuery :: String -> [String] -> Shell Line
fzfQuery description options =
  let fzfOptions = Te.pack <$> ["--ansi", "--prompt=" ++ description, "--height=30%"]
      selectables = T.toLines $ T.mconcat $ (return . Te.pack) <$> (L.intersperse "\n" options) in
  T.inproc (Te.pack "fzf") fzfOptions selectables

isFlake :: T.Shell T.Line
isFlake = fzfQuery "Is the input a flake? " ["yes", "no"]
