from ctypes import *
from vcx.common import do_call, create_cb
from vcx.api.connection import Connection
from vcx.api.vcx_stateful import VcxStateful
from typing import Optional

import json


class Credential(VcxStateful):
    """
    The object of the VCX API representing a Holder side in the credential issuance process.
    Assumes that pairwise connection between Issuer and Holder is already established.

    # State

    The set of object states and transitions depends on communication method is used.
    The communication method can be specified as config option on one of *_init function. The default communication method us `proprietary`.

    proprietary:
        VcxStateType::VcxStateRequestReceived - once `vcx_credential_create_with_offer` (create Credential object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `CRED_REQ` message) is called.

        VcxStateType::VcxStateAccepted - once `CRED` messages is received.
                                         use `vcx_credential_update_state` or `vcx_credential_update_state_with_message` functions for state updates.

    aries:
        VcxStateType::VcxStateRequestReceived - once `vcx_credential_create_with_offer` (create Credential object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `CredentialRequest` message) is called.

        VcxStateType::VcxStateAccepted - once `Credential` messages is received.
        VcxStateType::None - 1) once `ProblemReport` messages is received.
                                use `vcx_credential_update_state` or `vcx_credential_update_state_with_message` functions for state updates.
                             2) once `vcx_credential_reject` is called.

    # Transitions

    proprietary:
        VcxStateType::None - `vcx_credential_create_with_offer` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_credential_send_request` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `CRED` - VcxStateType::VcxStateAccepted

    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential
        VcxStateType::None - `vcx_credential_create_with_offer` - VcxStateType::VcxStateRequestReceived

        VcxStateType::VcxStateRequestReceived - `vcx_issuer_send_credential_offer` - VcxStateType::VcxStateOfferSent
        VcxStateType::VcxStateRequestReceived - `vcx_credential_reject` - VcxStateType::None

        VcxStateType::VcxStateOfferSent - received `Credential` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None
        VcxStateType::VcxStateOfferSent - `vcx_credential_reject` - VcxStateType::None

    # Messages

    proprietary:
        CredentialOffer (`CRED_OFFER`)
        CredentialRequest (`CRED_REQ`)
        Credential (`CRED`)

    aries:
        CredentialProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#propose-credential
        CredentialOffer - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#offer-credential
        CredentialRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#request-credential
        Credential - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0036-issue-credential#issue-credential
        ProblemReport - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0035-report-problem#the-problem-report-message-type
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
    """

    def __init__(self, source_id: str):
        VcxStateful.__init__(self, source_id)
        self._name = source_id
        self._cred_offer = None

    def __del__(self):
        self.release()
        self.logger.debug("Deleted {} obj: {}".format(Credential, self.handle))

    @property
    def cred_offer(self):
        return self._cred_offer

    @cred_offer.setter
    def cred_offer(self, x):
        self._cred_offer = x

    @staticmethod
    async def create(source_id: str, credential_offer: str):
        """
        Creates a credential with an offer.
        :param source_id: Institution's personal identification for the credential, should be unique.
        :param credential_offer: JSON string representing the offer used as the basis of creation.
        :return: A created credential
        Example:
        offer depends on communication method:
         proprietary:
            [{
               "msg_type": "CLAIM_OFFER",
               "version": "0.1",
               "to_did": "8XFh8yBzrpJQmNyZzgoTqB",
               "from_did": "8XFh8yBzrpJQmNyZzgoTqB",
               "libindy_offer": '{}',
               "credential_attrs": {
                  "address1": [
                     "101 Tela Lane"
                  ],
                  "address2": [
                     "101 Wilson Lane"
                  ],
                  "city": [
                     "SLC"
                  ],
                  "state": [
                     "UT"
                  ],
                  "zip": [
                     "87121"
                  ]
               },
               "schema_seq_no": 1487,
               "cred_def_id": "id1",
               "claim_name": "Credential",
               "claim_id": "defaultCredentialId",
               "msg_ref_id": None,
            }]
          aries:
            {
                "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential",
                "@id": "<uuid-of-offer-message>",
                "comment": "some comment",
                "credential_preview": <json-ld object>,
                "offers~attach": [
                    {
                        "@id": "libindy-cred-offer-0",
                        "mime-type": "application/json",
                        "data": {
                            "base64": "<bytes for base64>"
                        }
                    }
                ]
            }
        credential = await Credential.create(source_id, offer)
        """
        constructor_params = (source_id,)

        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_offer = c_char_p(json.dumps(credential_offer).encode('utf-8'))
        c_params = (c_source_id, c_offer, )

        return await Credential._create("vcx_credential_create_with_offer",
                                        constructor_params,
                                        c_params)

    @staticmethod
    async def parse_offer(offer: str):
        """
        Parse an Aries Credential Offer message
        :param offer: received credential offer message
        :return: credential offer info
        """
        if not hasattr(Credential.parse_offer, "cb"):
            Credential.parse_offer.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_offer = c_char_p(offer.encode('utf-8'))

        data = await do_call('vcx_credential_parse_offer',
                             c_offer,
                             Credential.parse_offer.cb)

        return json.loads(data.decode())

    @staticmethod
    async def create_with_msgid(source_id: str, connection: Connection, msg_id: str):
        """
        Create a credential based off of a known message id for a given connection.
        :param source_id: Institution's personal identification for the credential, should be unique.
        :param connection: connection to receive offer from
        :param msg_id: id of the message that contains the credential offer
        :return: A created credential
        Example:
        credential = await Credential.create_with_msgid(source_id, connection, msg_id)
        assert await credential.get_state() == State.RequestReceived
        """
        credential = Credential(source_id,)

        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_msg_id = c_char_p(msg_id.encode('utf-8'))
        c_connection_handle = c_uint32(connection.handle)

        if not hasattr(Credential.create_with_msgid, "cb"):
            Credential.create_with_msgid.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_uint32, c_char_p))

        credential.handle, cred_offer = await do_call('vcx_credential_create_with_msgid',
                                                      c_source_id,
                                                      c_connection_handle,
                                                      c_msg_id,
                                                      Credential.create_with_msgid.cb)

        credential.cred_offer = json.loads(cred_offer.decode())

        return credential

    @staticmethod
    async def accept_offer(source_id: str, credential_offer: str, connection: Connection):
        """
        Accept credential for the given offer.

        This function performs the following actions:
        1. Creates Credential state object that requests and receives a credential for an institution.
            (equal to `vcx_credential_create_with_offer` function).
        2. Prepares Credential Request and replies to the issuer.
            (equal to `vcx_credential_send_request` function).

        :param source_id: Institution's personal identification for the credential, should be unique.
        :param credential_offer: JSON string representing the offer used as the basis of creation.
        :param connection: A pairwise connection with the issuer.
        :return: credential object

        Example:
        offer depends on communication method:
         proprietary:
            [{
               "msg_type": "CLAIM_OFFER",
               "version": "0.1",
               "to_did": "8XFh8yBzrpJQmNyZzgoTqB",
               "from_did": "8XFh8yBzrpJQmNyZzgoTqB",
               "libindy_offer": '{}',
               "credential_attrs": {
                  "address1": [
                     "101 Tela Lane"
                  ],
                  "address2": [
                     "101 Wilson Lane"
                  ],
                  "city": [
                     "SLC"
                  ],
                  "state": [
                     "UT"
                  ],
                  "zip": [
                     "87121"
                  ]
               },
               "schema_seq_no": 1487,
               "cred_def_id": "id1",
               "claim_name": "Credential",
               "claim_id": "defaultCredentialId",
               "msg_ref_id": None,
            }]
          aries:
            {
                "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential",
                "@id": "<uuid-of-offer-message>",
                "comment": "some comment",
                "credential_preview": <json-ld object>,
                "offers~attach": [
                    {
                        "@id": "libindy-cred-offer-0",
                        "mime-type": "application/json",
                        "data": {
                            "base64": "<bytes for base64>"
                        }
                    }
                ]
            }
        credential = await Credential.accept_offer(source_id, offer, connection)
        """
        if not hasattr(Credential.accept_offer, "cb"):
            Credential.accept_offer.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_uint32, c_char_p))

        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_credential_offer = c_char_p(json.dumps(credential_offer).encode('utf-8'))
        c_connection_handle = c_uint32(connection.handle)

        credential_handle, credential_serialized = await do_call('vcx_credential_accept_credential_offer',
                                                                c_source_id,
                                                                c_credential_offer,
                                                                c_connection_handle,
                                                                Credential.accept_offer.cb)

        credential = Credential(source_id)
        credential.handle = credential_handle
        credential.serialized = json.loads(credential_serialized.decode())

        return credential

    @staticmethod
    async def deserialize(data: dict):
        """
        Create a credential object from a previously serialized credential object
        :param data: JSON data from a serialized object.
        :return: A created credential
        Example:
        credential = await Credential.create(source_id, offer)
        data = await credential.serialize()
        credential2 = await Credential.deserialize(data)
        """

        credential = await Credential._deserialize("vcx_credential_deserialize",
                                                   json.dumps(data),
                                                   data.get('data').get('source_id'))
        return credential

    @staticmethod
    async def get_offers(connection: Connection) -> dict:
        """
        Retrieves all pending credential offers for a given connection.
        :param connection: A connection to query for credential offers.
        :return: A list of dictionary objects representing offers from a given connection.
         "[[{"msg_type": "CREDENTIAL_OFFER","version": "0.1","to_did": "...","from_did":"...","credential": {"account_num": ["...."],"name_on_account": ["Alice"]},"schema_seq_no": 48,"issuer_did": "...","credential_name": "Account Certificate","credential_id": "3675417066","msg_ref_id": "ymy5nth"}]]"
        Example:
        credential = await Credential.create_with_msgid(source_id, connection, msg_id)
        offers = await credential.get_offers(connection)
        """
        if not hasattr(Credential.get_offers, "cb"):
            Credential.get_offers.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(connection.handle)

        data = await do_call('vcx_credential_get_offers',
                             c_connection_handle,
                             Credential.get_offers.cb)

        return json.loads(data.decode())

    async def serialize(self) -> dict:
        """
        Serializes the crednetial object for storage and later deserialization.
        :return: dict representing the object
        Example:
        credential = await Credential.create_with_msgid(source_id, connection, msg_id)
        data = credential.serialzie()
        """
        return await self._serialize(Credential, 'vcx_credential_serialize')

    async def update_state(self) -> int:
        """
        Query the agency for the received messages.
        Checks for any messages changing state in the credential object and updates the state attribute.
        If it detects a credential it will store the credential in the wallet.

        Example:
        credential = await Credential.create(source_id, offer)
        credential.update_state()
        :return:
        """
        return await self._update_state(Credential, 'vcx_credential_update_state')

    async def update_state_with_message(self, message: str) -> int:

        """
        Update the state of the credential based on the given message.
        Example:
        proof = await Proof.create(source_id)
        assert await proof.update_state_with_message(message) == State.Accepted
        :param message: message to process for state changes
        :return Current state of the credential
        """
        return await self._update_state_with_message(Credential, message, 'vcx_credential_update_state_with_message')

    async def get_state(self) -> int:
        """
        Get the current state of the credential object
        :return: credential state of the object. Possible states:
                 2 - Request Sent
                 3 - Request Received
                 4 - Accepted

        Example:
        credential = await Credential.create(source_id, offer)
        credential.update_state()
        state = credential.get_state()
        """
        return await self._get_state(Credential, 'vcx_credential_get_state')

    def release(self) -> None:
        """
        Used to release memory associated with this object, used by the c library.
        :return:
        """
        self._release(Credential, 'vcx_credential_release')

    async def send_request(self, connection: Optional[Connection], payment_handle: int):
        """
        Approves the credential offer and submits a credential request. The result will be a credential stored in the prover's wallet.
        :param connection: connection to send credential request
                              Pass `null` in order to reply on ephemeral/connectionless credential offer
                              Ephemeral/Connectionless Credential Offer contains `~server` decorator

        :param payment_handle: currently unused
        :return:
        Example:
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        credential = await Credential.create(source_id, offer)
        await credential.send_request(connection, 0)
        """
        if not hasattr(Credential.send_request, "cb"):
            self.logger.debug("vcx_credential_send_request: Creating callback")
            Credential.send_request.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_credential_handle = c_uint32(self.handle)
        c_connection_handle = c_uint32(connection.handle if connection else 0)
        c_payment = c_uint32(payment_handle)

        await do_call('vcx_credential_send_request',
                      c_credential_handle,
                      c_connection_handle,
                      c_payment,
                      Credential.send_request.cb)

    async def get_request_msg(self, my_pw_did: str, their_pw_did: str, payment_handle: int):
        """
        Approves the credential offer and gets the credential request message
        :param my_pw_did: my pairwaise did
        :param their_pw_did: pairwaise did of other side
        :param payment_handle: currently unused
        :return:
        Example:
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        credential = await Credential.create(source_id, offer)
        await credential.get_request_msg(my_pw_did, their_pw_did, 0)
        """
        if not hasattr(Credential.get_request_msg, "cb"):
            self.logger.debug("vcx_credential_get_request_msg: Creating callback")
            Credential.get_request_msg.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_credential_handle = c_uint32(self.handle)
        c_my_pw_did = c_char_p(my_pw_did.encode('utf-8'))
        c_their_pw_did = c_char_p(their_pw_did.encode('utf-8'))
        c_payment = c_uint32(payment_handle)

        msg = await do_call('vcx_credential_get_request_msg',
                      c_credential_handle,
                      c_my_pw_did,
                      c_their_pw_did,
                      c_payment,
                      Credential.get_request_msg.cb)

        return json.loads(msg.decode())

    async def get_credential(self):
        """
        Retrieve information about a stored credential in user's wallet,
        including credential id and the credential itself.

        :return:
        Example:
        credential = await Credential.create(source_id, offer)
        await credential.get_credential()
        """
        if not hasattr(Credential.get_credential, "cb"):
            self.logger.debug("vcx_get_credential: Creating callback")
            Credential.get_credential.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_credential_handle = c_uint32(self.handle)

        msg = await do_call('vcx_get_credential',
                      c_credential_handle,
                      Credential.get_credential.cb)

        return json.loads(msg.decode())

    async def get_payment_info(self):
        """
        Retrieve Payment Transaction Information for this Credential. Typically this will include
        how much payment is requried by the issuer, which needs to be provided by the prover, before the issuer will
        issue the credential to the prover. Ideally a prover would want to know how much payment is being asked before
        submitting the credential request (which triggers the payment to be made).
        Example:
        info = credential.get_payment_info()
        :return: payment information
             {
                 "payment_required":"one-time",
                 "payment_addr":"pov:null:OsdjtGKavZDBuG2xFw2QunVwwGs5IB3j",
                 "price":1
             }
        """
        if not hasattr(Credential.get_payment_info, "cb"):
            self.logger.debug("vcx_credential_get_payment_info: Creating callback")
            Credential.get_payment_info.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_credential_handle = c_uint32(self.handle)
        data = await do_call('vcx_credential_get_payment_info',
                      c_credential_handle,
                      Credential.get_payment_info.cb)
        return json.loads(data.decode())

    async def get_payment_txn(self):
        """
        Retrieve the payment transaction associated with this credential. This can be used to get the txn that
        was used to pay the issuer from the prover.  This could be considered a receipt of payment from the payer to
        the issuer.
        :return: payment transaction
        {
            "amount":25,
            "inputs":[
                "pay:null:1_3FvPC7dzFbQKzfG"
            ],
            "outputs":[
                {"recipient":"pay:null:FrSVC3IrirScyRh","amount":5,"extra":null}
            ]
        }
        Example:
        txn = credential.get_payment_txn()
        """
        if not hasattr(Credential.get_payment_txn, "cb"):
            self.logger.debug("vcx_credential_get_payment_txn: Creating callback")
            Credential.get_payment_txn.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_credential_handle = c_uint32(self.handle)

        payment_txn = await do_call('vcx_credential_get_payment_txn',
                      c_credential_handle,
                      Credential.get_payment_txn.cb)

        return json.loads(payment_txn.decode())

    async def reject(self, connection: Connection, comment: Optional[str] = None):
        """
        Send a Credential rejection to the connection.
        It can be called once Credential Offer or Credential messages are received.

        Note that this function can be used for `aries` communication protocol.
        In other cases it returns ActionNotSupported error.

        :param connection: connection to submit credential reject
        :param comment: (Optional) human-friendly message to insert into Reject message.
        :return:

        Example:
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        credential = await Credential.create(source_id, offer)
        await credential.reject(connection, None)
        """
        if not hasattr(Credential.reject, "cb"):
            self.logger.debug("vcx_credential_reject: Creating callback")
            Credential.reject.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_credential_handle = c_uint32(self.handle)
        c_connection_handle = c_uint32(connection.handle)
        c_comment = c_char_p(comment.encode('utf-8')) if comment is not None else None

        await do_call('vcx_credential_reject',
                      c_credential_handle,
                      c_connection_handle,
                      c_comment,
                      Credential.reject.cb)

    async def get_presentation_proposal(self):
        """
        Build Presentation Proposal message for revealing Credential data.

        Presentation Proposal is an optional message that can be sent by the Prover to the Verifier to
        initiate a Presentation Proof process.

        Presentation Proposal Format:
            https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof#propose-presentation

        EXPERIMENTAL

        :return:
        Example:
        credential = await Credential.create(source_id, offer)
        await credential.get_presentation_proposal()
        """
        if not hasattr(Credential.get_presentation_proposal, "cb"):
            self.logger.debug("vcx_credential_get_presentation_proposal_msg: Creating callback")
            Credential.get_presentation_proposal.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_credential_handle = c_uint32(self.handle)

        msg = await do_call('vcx_credential_get_presentation_proposal_msg',
                      c_credential_handle,
                      Credential.get_presentation_proposal.cb)

        return json.loads(msg.decode())

    async def delete(self):
        """
        Delete a Credential associated with the state object from the Wallet and release handle of the state object.

        :return:
        Example:
        credential = await Credential.create(source_id, offer)
        await credential.delete()
        """
        if not hasattr(Credential.delete, "cb"):
            self.logger.debug("vcx_delete_credential: Creating callback")
            Credential.delete.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_credential_handle = c_uint32(self.handle)

        await do_call('vcx_delete_credential',
                      c_credential_handle,
                      Credential.delete.cb)

    async def get_problem_report(self) -> Optional[str]:
        """
        Get Problem Report message for object in Failed or Rejected state.
        :return: Problem Report as JSON string or null
        """

        if not hasattr(Credential.get_problem_report, "cb"):
            self.logger.debug("vcx_credential_get_problem_report: Creating callback")
            Credential.get_problem_report.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(self.handle)
        result = await do_call('vcx_credential_get_problem_report',
                               c_connection_handle,
                               Credential.get_problem_report.cb)

        self.logger.debug("vcx_credential_get_problem_report completed")
        return result.decode() if result else None

    async def vcx_credential_get_info(self) -> Optional[str]:
        """
        Retrieve information about a stored credential.
        :return: Credential Info as JSON string or null
           {
               "referent": string, // cred_id in the wallet
               "attrs": {"key1":"raw_value1", "key2":"raw_value2"},
               "schema_id": string,
               "cred_def_id": string,
               "rev_reg_id": Optional<string>,
               "cred_rev_id": Optional<string>
           }
        """

        if not hasattr(Credential.vcx_credential_get_info, "cb"):
            self.logger.debug("vcx_credential_get_info: Creating callback")
            Credential.vcx_credential_get_info.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(self.handle)
        result = await do_call('vcx_credential_get_info',
                               c_connection_handle,
                               Credential.vcx_credential_get_info.cb)

        self.logger.debug("vcx_credential_get_info completed")
        return result.decode() if result else None







