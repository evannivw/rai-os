# Whole-set integration fixture

The goal events use the three exact required sets in review_pack/v1. They do not
implement that fixture's complete release/outage/budget contract. Time is measured
from coordinator startup. Events are scripted, and no sensor data is claimed.

`replay.json` is an explicitly offline decision sequence for a fresh QEMU boot.
It deliberately chooses notes before brief rather than simply following sorted
key order. It must not be described as model inference or an optimal schedule.
Generations 1, 5 and 7 assume the preceding jobs completed before each new goal.
Live Meta chooses its own order and may decline or respond after a goal change.

Only requirements/byte lengths and current guest readiness reach the model. The
future schedule, record bodies and reference outputs do not. The independently
checked output is generated inside RAI-OS, not returned by the model.
