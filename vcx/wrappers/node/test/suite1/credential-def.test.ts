import '../module-resolver-helper'

import { assert } from 'chai'
import { credentialDefCreate, credentialDefCreateWithId, credentialDefPrepareForEndorser } from 'helpers/entities'
import { initVcxTestMode, shouldThrow } from 'helpers/utils'
import { CredentialDef, CredentialDefPaymentManager, CredentialDefState, VCXCode } from 'src'

describe('CredentialDef:', () => {
  before(() => initVcxTestMode())

  describe('create:', () => {
    it('success', async () => {
      await credentialDefCreate()
    })
  })

  describe('createWithId:', () => {
    it('success', async () => {
      await credentialDefCreateWithId()
    })
  })

  describe('serialize:', () => {
    it('success', async () => {
      const credentialDef = await credentialDefCreate()
      const serialized = await credentialDef.serialize()
      assert.ok(serialized)
      assert.property(serialized, 'version')
      assert.property(serialized, 'data')
      const { data, version } = serialized
      assert.ok(data)
      assert.ok(version)
      assert.equal(data.source_id, credentialDef.sourceId)
    })

    it('throws: not initialized', async () => {
      const credentialDef = new CredentialDef(null as any, {} as any)
      const error = await shouldThrow(() => credentialDef.serialize())
      assert.equal(error.vcxCode, VCXCode.INVALID_CREDENTIAL_DEF_HANDLE)
    })

  })

  describe('deserialize:', () => {
    it('success', async () => {
      const credentialDef1 = await credentialDefCreate()
      const data1 = await credentialDef1.serialize()
      const credentialDef2 = await CredentialDef.deserialize(data1)
      assert.equal(credentialDef2.sourceId, credentialDef1.sourceId)
      const data2 = await credentialDef2.serialize()
      assert.deepEqual(data1, data2)
    })

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () => CredentialDef.deserialize({ data: { source_id: 'Invalid' } } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON)
    })
  })

  describe('getCredDefId:', () => {
    it('success', async () => {
      const credentialDef = await credentialDefCreate()
      assert.equal(await credentialDef.getCredDefId(), '2hoqvcwupRTUNkXn6ArYzs:3:CL:2471')
    })

    it('throws: not initialized', async () => {
      const credentialDef = new CredentialDef(null as any, {} as any)
      const error = await shouldThrow(() => credentialDef.getCredDefId())
      assert.equal(error.vcxCode, VCXCode.INVALID_CREDENTIAL_DEF_HANDLE)
    })
  })

  describe('prepareForEndorser:', () => {
    it('success', async () => {
      await credentialDefPrepareForEndorser()
    })
  })

  describe('updateState:', () => {
    it(`success`, async () => {
      const credentialDef = await credentialDefPrepareForEndorser()
      assert.equal(await credentialDef.getState(), CredentialDefState.Built)
      await credentialDef.updateState()
      assert.equal(await credentialDef.getState(), CredentialDefState.Published)
    })
  })
})
