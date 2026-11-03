#!/usr/bin/env python3
"""One fixed user objective; controlled assignment context triggers fresh decisions."""
import sys
sys.dont_write_bytecode=True
from semantic_session import ROOT,main
FIXTURE=ROOT/'fixtures/context_session/v1'
if __name__=='__main__':main(FIXTURE,contextual=True)
