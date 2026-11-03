#!/usr/bin/env python3
"""Verify constant-goal context policy, provider boundary and actual guest continuity."""
import argparse
import sys
sys.dont_write_bytecode=True
from context_session import FIXTURE
from verify_semantic_session import verify
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--debug',action='store_true');p.add_argument('--graphics',action='store_true');a=p.parse_args()
    verify(a.debug,a.graphics,FIXTURE,contextual=True)
