import '../module-resolver-helper'

import { assert } from 'chai'
import {
  connectionCreateConnect,
  credentialCreateWithMsgId,
  credentialCreateWithOffer,
  dataCredentialCreateWithMsgId,
  dataCredentialCreateWithOffer
} from 'helpers/entities'
import { initVcxTestMode, shouldThrow } from 'helpers/utils'
import { Credential, StateType, VCXCode, VCXMock, VCXMockMessage } from 'src'

describe('Credential:', () => {
  before(() => initVcxTestMode())

  describe('create:', () => {
    it('success', async () => {
      await credentialCreateWithOffer()
    })

    it('throws: missing sourceId', async () => {
      const { sourceId, ...data } = await dataCredentialCreateWithOffer()
      const error = await shouldThrow(() => Credential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing offer', async () => {
      const { offer, ...data } = await dataCredentialCreateWithOffer()
      const error = await shouldThrow(() => Credential.create(data as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    // Enable when we start utilizing connection prop
    it.skip('throws: missing connection', async () => {
      const { connection, ...data } = await dataCredentialCreateWithOffer()
      const error = await shouldThrow(() => Credential.create({ connection: {} as any, ...data }))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: invalid offer', async () => {
      const { offer, ...data } = await dataCredentialCreateWithOffer()
      const error = await shouldThrow(() => Credential.create({ offer: 'invalid', ...data }))
      assert.equal(error.vcxCode, VCXCode.INVALID_CREDENTIAL_OFFER)
    })
  })

  describe('createWithMsgId:', () => {
    it('success', async () => {
      await credentialCreateWithMsgId()
    })

    it('throws: missing sourceId', async () => {
      const { connection, msgId } = await dataCredentialCreateWithMsgId()
      const error = await shouldThrow(() => Credential.createWithMsgId({ connection, msgId } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing offer', async () => {
      const { connection, sourceId } = await dataCredentialCreateWithMsgId()
      const error = await shouldThrow(() => Credential.createWithMsgId({ connection, sourceId } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_OPTION)
    })

    it('throws: missing connection', async () => {
      const { connection, ...data } = await dataCredentialCreateWithMsgId()
      const error = await shouldThrow(() => Credential.createWithMsgId(data as any))
      assert.equal(error.vcxCode, VCXCode.UNKNOWN_ERROR)
    })

    it('throws: missing connection handle', async () => {
      const { connection, ...data } = await dataCredentialCreateWithMsgId()
      const error = await shouldThrow(() => Credential.createWithMsgId({ connection: {} as any, ...data }))
      assert.equal(error.vcxCode, VCXCode.INVALID_CONNECTION_HANDLE)
    })
  })

  describe('serialize:', () => {
    it('success', async () => {
      const credential = await credentialCreateWithOffer()
      const serialized = await credential.serialize()
      assert.ok(serialized)
      assert.property(serialized, 'version')
      assert.property(serialized, 'data')
      const { data, version } = serialized
      assert.ok(data)
      assert.ok(version)
      assert.equal(data.source_id, credential.sourceId)
    })

    it('throws: not initialized', async () => {
      const credential = new Credential(null as any)
      const error = await shouldThrow(() => credential.serialize())
      assert.equal(error.vcxCode, VCXCode.INVALID_CREDENTIAL_HANDLE)
    })

  })

  describe('deserialize:', () => {
    it('success', async () => {
      const credential1 = await credentialCreateWithOffer()
      const data1 = await credential1.serialize()
      const credential2 = await Credential.deserialize(data1)
      assert.equal(credential2.sourceId, credential1.sourceId)
      const data2 = await credential2.serialize()
      assert.deepEqual(data1, data2)
    })

    it('throws: incorrect data', async () => {
      const error = await shouldThrow(async () => Credential.deserialize({
        data: { source_id: 'Invalid' } } as any))
      assert.equal(error.vcxCode, VCXCode.INVALID_JSON)
    })
  })

  describe('updateState:', () => {
    it(`returns ${StateType.None}: not initialized`, async () => {
      const credential = new Credential(null as any)
      const error = await shouldThrow(() => credential.updateState())
      assert.equal(error.vcxCode, VCXCode.INVALID_CREDENTIAL_HANDLE)
    })

    it(`returns ${StateType.RequestReceived}: created`, async () => {
      const credential = await credentialCreateWithOffer()
      await credential.updateState()
      assert.equal(await credential.getState(), StateType.RequestReceived)
    })
  })

  describe('sendRequest:', () => {
    it('success: with offer', async () => {
      const data = await dataCredentialCreateWithOffer()
      const credential = await credentialCreateWithOffer(data)
      await credential.sendRequest({ connection: data.connection, payment: 0 })
      assert.equal(await credential.getState(), StateType.OfferSent)
    })

    // it('success: with message id', async () => {
    //   const data = await dataCredentialCreateWithMsgId()
    //   const credential = await credentialCreateWithMsgId(data)
    //   await credential.sendRequest({ connection: data.connection, payment: 0 })
    //   assert.equal(await credential.getState(), StateType.OfferSent)
    // })

    it('success: get request message', async () => {
      const data = await dataCredentialCreateWithOffer()
      const credential = await credentialCreateWithOffer(data)
      const pwDid = await data.connection.getPwDid()
      const msg = await credential.getRequestMessage({ myPwDid: pwDid, payment: 0 })
      assert(msg.length > 0)
    })

    it('success: issued', async () => {
      const data = await dataCredentialCreateWithOffer()
      const credential = await credentialCreateWithOffer(data)
      await credential.sendRequest({ connection: data.connection, payment: 0 })
      assert.equal(await credential.getState(), StateType.OfferSent)
      VCXMock.setVcxMock(VCXMockMessage.CredentialResponse)
      VCXMock.setVcxMock(VCXMockMessage.UpdateIssuerCredential)
      await credential.updateState()
      assert.equal(await credential.getState(), StateType.Accepted)
    })
  })

  describe('getOffers:', () => {
    it('success', async () => {
      const connection = await connectionCreateConnect()
      const offers = await Credential.getOffers(connection)
      assert.ok(offers)
      assert.ok(offers.length)
      const offer = offers[0]
      await credentialCreateWithOffer({
        connection,
        offer: JSON.stringify(offer),
        sourceId: 'credentialGetOffersTestSourceId'
      })
    })
  })

  describe('acceptOffer:', () => {
    it('success: accept credential offer', async () => {
      const data = await dataCredentialCreateWithOffer()
      const credential = await Credential.acceptOffer(data)
      assert.equal(await credential.getState(), StateType.OfferSent)
    })
  })

  describe('rejectCredentail:', () => {
    it('success: reject credential offer', async () => {
      const data = await dataCredentialCreateWithOffer()
      const credential = await credentialCreateWithOffer(data)
      const error = await shouldThrow(() => credential.reject({ connection: data.connection }))
      assert.equal(error.vcxCode, VCXCode.ACTION_NOT_SUPPORTED)
    })
  })
})
