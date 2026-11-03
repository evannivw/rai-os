"""Explicit offline fixtures. Never consulted by the live Meta path."""
import copy

CREATE='Create an inspection app with equipment, notes, a done checkbox and a pending view.'
EVOLVE='Add a required integer priority field with default 1. Preserve all existing fields, records and the pending view.'
SPEC=dict(id='inspection',title='Field inspections',capability='records.local',fields=[
 dict(key='equipment',label='Equipment',kind='text',required=True,default=''),
 dict(key='notes',label='Notes',kind='text',required=False,default=''),
 dict(key='done',label='Done',kind='check',required=True,default='false')],filter_key='done',filter_label='Pending')

TABLE='Create an expense app with item, integer amount in cents, paid checkbox and an unpaid view. I want a table and totals.'
TABLE_SPEC=dict(id='expenses',title='Expenses',capability='records.local',fields=[
 dict(key='item',label='Item',kind='text',required=True,default=''),
 dict(key='amount',label='Amount (cents)',kind='number',required=True,default='0'),
 dict(key='paid',label='Paid',kind='check',required=True,default='false')],filter_key='paid',filter_label='Unpaid')

BRIGHTNESS='Create an app to adjust the RAI-OS display brightness from 20 to 100 percent, with a default of 50.'
BRIGHTNESS_SPEC=dict(id='brightness',title='Screen brightness',capability='display.brightness',fields=[dict(key='level',label='Brightness %',kind='number',required=True,default='50')],filter_key='',filter_label='')

def propose(r):
    rid=r["request_id"]
    spec=None
    if r['goal']==CREATE and r['target'] is None:spec=copy.deepcopy(SPEC)
    if r['goal']==TABLE and r['target'] is None:spec=copy.deepcopy(TABLE_SPEC)
    if r['goal']==BRIGHTNESS and r['target'] is None:spec=copy.deepcopy(BRIGHTNESS_SPEC)
    if r['goal']==EVOLVE and r['target'] is not None:
        spec=copy.deepcopy(r['target']);spec['fields'].append(dict(key='priority',label='Priority',kind='number',required=True,default='1'))
    plan=dict(request_id=rid,base_revision=r['base_revision'],decision='install' if spec else 'clarify',spec=spec,reason='Scripted offline proposal. Review before installing.' if spec else 'Offline replay supports the documented prompts only. Select Meta for your own app.')
    return plan
