#!/bin/sh

#  fetch-deps.sh
#  FlowBetween

if [ ! -d SVGKit ]; then
    git clone https://github.com/SVGKit/SVGKit.git
fi

cd SVGKit
git checkout 3.x

