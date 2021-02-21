{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE ExtendedDefaultRules #-}
{-# OPTIONS_GHC -fno-warn-type-defaults #-}
module Main where

import FzfLib as Fzf
--import Data.Text as T
--import Data.Text.IO as TIO
--import Shelly.Pipe as S
import Turtle as T

--supportedPackages = [ python ];

-- echo "a\nb"
-- fzf --ansi --prompt='Choose a package' --height=30%

main :: IO ()
main =
  T.view $ Fzf.fzfQuery "pick a thing bro: " ["a", "b", "c"] <> isFlake

isFlake :: T.Shell T.Line
isFlake =
  Fzf.fzfQuery "Is the input a flake? " ["yes", "no"]

