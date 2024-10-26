#!/bin/bash

cutechess-cli \
  -engine conf=uci-pvs \
  -engine conf=uci-negamax \
  -each tc=40/60 timemargin=300 \
  -games 100 \
  -rounds 1 \
  -repeat \
  -concurrency 1 \
  -pgnout tournament_results.pgn \
  -recover \
  -wait 5000 \
  -draw movenumber=40 movecount=8 score=10 \
  -resign movecount=3 score=400 \
  -debug
