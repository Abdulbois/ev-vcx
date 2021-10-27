import '../module-resolver-helper'

import { assert } from 'chai'
import { initVcxTestMode, shouldThrow } from 'helpers/utils'
import { shutdownVcx, VCXCode, Wallet } from 'src'

const WALLET_RECORD = {
  id: 'RecordId',
  tags: {},
  type_: 'TestType',
  value: 'RecordValue'
}

const OPTIONS = {
  retrieveTags: false,
  retrieveType: true,
  retrieveValue: true
}
const QUERY_JSON = { tagName1: 'str1' }

const UPDATE_WALLET_RECORD = {
  id: 'RecordId',
  type_: 'TestType',
  value: 'RecordValueNew'
}
const UPDATE_WALLET_TAGS = {
  id: 'RecordId',
  tags: {},
  type_: 'TestType',
  value: ''
}

const TAGS = '{"tagName1":"str1","tagName2":"5","tagName3":"12"}'

const SEARCHED_RECORD = {
  id: 'RecordId',
  tags: TAGS,
  type: null,
  value: 'RecordValue'
}

describe('Wallet:', () => {
  before(() => initVcxTestMode())

  describe('records:', () => {
    it('success', async () => {
      await Wallet.addRecord(WALLET_RECORD)
      await Wallet.getRecord({ type: WALLET_RECORD.type_, id: WALLET_RECORD.id, options: OPTIONS })
      await Wallet.updateRecordValue(UPDATE_WALLET_RECORD)
      await Wallet.updateRecordTags(UPDATE_WALLET_TAGS)
      await Wallet.addRecordTags(UPDATE_WALLET_TAGS)
      await Wallet.deleteRecordTags(WALLET_RECORD, { tagList: ['one', 'two'] })
      await Wallet.deleteRecord({ type: WALLET_RECORD.type_, id: WALLET_RECORD.id })
      const searchHandle = await Wallet.openSearch({
        options: 'null',
        queryJson: JSON.stringify(QUERY_JSON),
        type: WALLET_RECORD.type_
      })
      assert(searchHandle === 1)
      const retrievedRecords = JSON.parse(await Wallet.searchNextRecords(searchHandle, { count: 1 }))
      assert.deepEqual(retrievedRecords, SEARCHED_RECORD)
      await Wallet.closeSearch(searchHandle)
    })
  })

  describe('import:', () => {
    it('throws: libindy error', async () => {
      let config = '{"wallet_name":"name","wallet_key":"","exported_wallet_path":"","backup_key":""}'
      let error = await shouldThrow(async () => Wallet.import(config))
      assert.equal(error.vcxCode, VCXCode.IO_ERROR)
      shutdownVcx(false)

      config = '{"wallet_key":"","exported_wallet_path":"","backup_key":""}'
      error = await shouldThrow(async () => Wallet.import(config))
      assert.equal(error.vcxCode, VCXCode.INVALID_WALLET_IMPORT_CONFIG)
      shutdownVcx(false)

      config = '{"wallet_name":"","exported_wallet_path":"","backup_key":""}'
      error = await shouldThrow(async () => Wallet.import(config))
      assert.equal(error.vcxCode, VCXCode.INVALID_WALLET_IMPORT_CONFIG)
      shutdownVcx(false)

      config = '{"wallet_name":"","wallet_key":"","backup_key":""}'
      error = await shouldThrow(async () => Wallet.import(config))
      assert.equal(error.vcxCode, VCXCode.INVALID_WALLET_IMPORT_CONFIG)
      shutdownVcx(false)

      config = '{"wallet_name":"","wallet_key":"","exported_wallet_path":""}'
      error = await shouldThrow(async () => Wallet.import(config))
      assert.equal(error.vcxCode, VCXCode.INVALID_WALLET_IMPORT_CONFIG)
    })
  })

  describe('export:', () => {
    it('throws: libindy error', async () => {
      const error = await shouldThrow(async () => Wallet.export('/tmp/foobar.wallet', 'key_for_wallet'))
      assert.equal(error.vcxCode, VCXCode.INVALID_WALLET_HANDLE)
    })
  })
})
