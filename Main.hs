{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE ExtendedDefaultRules #-}
{-# OPTIONS_GHC -fno-warn-type-defaults #-}
module Main where

import FzfLib as Fzf
import Data.Text as TE
--import Data.Text.IO as TIO
--import Shelly.Pipe as S
import Turtle as T
--import Hnix as H
import Nix.Parser as P
import Nix.Pretty as PP

--supportedPackages = [ python ];

-- echo "a\nb"
-- fzf --ansi --prompt='Choose a package' --height=30%
nixText = TE.pack " \
\ { \
\   inputs.nixpkgs.url = \"github:NixOS/nixpkgs/nixos-20.09\"; \
\   inputs.flake-utils.url = \"github:numtide/flake-utils\"; \
\ } "

main :: IO ()
main =
  --parsing the expression
  case P.parseNixText nixText of
    --printing the result
    Success r -> putStrLn $ (show $ r)
    --Success r -> putStrLn $ show $ PP.prettyNix r
    Failure _ -> putStrLn "failure to parse nix expression"

  --T.view $ Fzf.fzfQuery "pick a thing bro: " ["a", "b", "c"] <> Fzf.isFlake


