from ctypes import *
from typing import Optional

from vcx.common import do_call, create_cb
from vcx.api.connection import Connection
from vcx.api.vcx_stateful import VcxStateful

import json

class Proof(VcxStateful):
    """
    The object of the VCX API representing a Verifier side in the credential presentation process.
    Assumes that pairwise connection between Verifier and Prover is already established.

    # State

    The set of object states and transitions depends on communication method is used.
    The communication method can be specified as config option on one of *_init function. The default communication method us `proprietary`.

    proprietary:
        VcxStateType::VcxStateInitialized - once `vcx_proof_create` (create Proof object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `PROOF_REQ` message) is called.

        VcxStateType::VcxStateAccepted - once `PROOF` messages is received.
                                         use `vcx_proof_update_state` or `vcx_proof_update_state_with_message` functions for state updates.

    aries:
        VcxStateType::VcxStateInitialized - once `vcx_proof_create` (create Proof object) is called.

        VcxStateType::VcxStateOfferSent - once `vcx_credential_send_request` (send `PresentationRequest` message) is called.

        VcxStateType::VcxStateAccepted - once `Presentation` messages is received.
        VcxStateType::None - once `ProblemReport` messages is received.
        VcxStateType::None - once `PresentationProposal` messages is received.
        VcxStateType::None - on `Presentation` validation failed.
                                                use `vcx_proof_update_state` or `vcx_proof_update_state_with_message` functions for state updates.

    # Transitions

    proprietary:
        VcxStateType::None - `vcx_proof_create` - VcxStateType::VcxStateInitialized

        VcxStateType::VcxStateInitialized - `vcx_credential_send_request` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `PROOF` - VcxStateType::VcxStateAccepted

    aries: RFC - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
        VcxStateType::None - `vcx_proof_create` - VcxStateType::VcxStateInitialized

        VcxStateType::VcxStateInitialized - `vcx_credential_send_request` - VcxStateType::VcxStateOfferSent

        VcxStateType::VcxStateOfferSent - received `Presentation` - VcxStateType::VcxStateAccepted
        VcxStateType::VcxStateOfferSent - received `PresentationProposal` - VcxStateType::None
        VcxStateType::VcxStateOfferSent - received `ProblemReport` - VcxStateType::None

    # Messages

    proprietary:
        ProofRequest (`PROOF_REQ`)
        Proof (`PROOF`)

    aries:
        PresentationRequest - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#request-presentation
        Presentation - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#presentation
        PresentationProposal - https://github.com/hyperledger/aries-rfcs/tree/7b6b93acbaf9611d3c892c4bada142fe2613de6e/features/0037-present-proof#propose-presentation
        Ack - https://github.com/hyperledger/aries-rfcs/tree/master/features/0015-acks#explicit-acks
    """

    def __init__(self, source_id: str):
        VcxStateful.__init__(self, source_id)
        self._proof_state = 0

    def __del__(self):
        self.release()
        self.logger.debug("Deleted {} obj: {}".format(Proof, self.handle))

    @property
    def proof_state(self):
        return self._proof_state

    @proof_state.setter
    def proof_state(self, x):
        self._proof_state = x

    @staticmethod
    async def create(source_id: str, name: str, requested_attrs: list, revocation_interval: dict,
                     requested_predicates=None):
        """
         Create a new Proof object that requests a proof for an enterprise
        :param source_id: Enterprise's personal identification for the proof, should be unique.
        :param name: Name of the Proof
        :param requested_attrs: Attributes associated with the Proof
           {
               "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
               "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
                                                    // NOTE: should either be "name" or "names", not both and not none of them.
                                                    // Use "names" to specify several attributes that have to match a single credential.
               "restrictions":  (filter_json) {
                  "schema_id": string, (Optional)
                  "schema_issuer_did": string, (Optional)
                  "schema_name": string, (Optional)
                  "schema_version": string, (Optional)
                  "issuer_did": string, (Optional)
                  "cred_def_id": string, (Optional)
              },
               "non_revoked": {
                   "from": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
                   "to": Optional<(u64)>
                       //Requested time represented as a total number of seconds from Unix Epoch, Optional
               }
           }
        :param requested_predicates: Predicates associated with the Proof
           { // set of requested predicates
              "name": attribute name, (case insensitive and ignore spaces)
              "p_type": predicate type (">=", ">", "<=", "<")
              "p_value": int predicate value
              "restrictions": Optional<filter_json>, // see above
              "non_revoked": Optional<{
                  "from": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
                  "to": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
              }>
           }
        :param revocation_interval: interval applied to all requested attributes indicating when the claim must be valid (NOT revoked)
        Example:
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "names":["name", "male"], "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        revocation_interval = {"from": 1, "to": 2}  // Both values are optional
        proof = await Proof.create(source_id, name, requested_attrs)
        :return: Proof Object
        """
        if requested_predicates is None:
            requested_predicates = []
        constructor_params = (source_id,)

        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_name = c_char_p(name.encode('utf-8'))
        c_req_predicates = c_char_p(json.dumps(requested_predicates).encode('utf-8'))
        c_req_attrs = c_char_p(json.dumps(requested_attrs).encode('utf-8'))
        c_revocation_interval = c_char_p(json.dumps(revocation_interval).encode('utf-8'))
        c_params = (c_source_id, c_req_attrs, c_req_predicates, c_revocation_interval, c_name)

        return await Proof._create("vcx_proof_create",
                                   constructor_params,
                                   c_params)

    @staticmethod
    async def create_with_proposal(source_id: str, presentation_proposal: str, name: str):
        """
         Create a new Proof object based on the given Presentation Proposal message

        :param source_id: Enterprise's personal identification for the proof, should be unique.
        :param name: Name of the Proof
        :param presentation_proposal: Message sent by the Prover to the verifier to initiate a proof presentation process:
        {
            "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/propose-presentation",
            "@id": "<uuid-propose-presentation>",
            "comment": "some comment",
            "presentation_proposal": {
                "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/presentation-preview",
                "attributes": [
                    {
                        "name": "<attribute_name>", - name of the attribute.
                        "cred_def_id": "<cred_def_id>", - maps to the credential definition identifier of the credential with the current attribute
                        "mime-type": Optional"<type>", - optional type of value. if mime-type is missing (null), then value is a string.
                        "value": "<value>", - value of the attribute to reveal in presentation
                    },
                    // more attributes
                  ],
                 "predicates": [
                    {
                        "name": "<attribute_name>", - name of the attribute.
                        "cred_def_id": "<cred_def_id>", - maps to the credential definition identifier of the credential with the current attribute
                        "predicate": "<predicate>", - predicate operator: "<", "<=", ">=", ">"
                        "threshold": <threshold> - threshold value for the predicate.
                    },
                    // more predicates
                ]
            }
        }

        Example:
        name = "proof name"
        presentation_proposal = {"@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/presentation", "@id": "<uuid-presentation>", "comment": "somecomment", "presentation_proposal": {"@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/presentation-preview", "attributes":[{"name": "account", "cred_def_id": "BzCbsNYhMrjHiqZDTUASHg:3:CL:1234:tag", "value": "12345678","referent": "0"}, {"name": "streetAddress", "cred_def_id": "BzCbsNYhMrjHiqZDTUASHg:3:CL:1234:tag","value": "123MainStreet", "referent": "0"},], "predicates": []}}
        proof = await Proof.create_with_proposal(source_id, json.dumps(presentation_proposal), name)
        :return: Proof Object
        """
        constructor_params = (source_id,)

        c_source_id = c_char_p(source_id.encode('utf-8'))
        c_name = c_char_p(name.encode('utf-8'))
        c_presentation_proposal = c_char_p(presentation_proposal.encode('utf-8'))
        c_params = (c_source_id, c_presentation_proposal, c_name)

        return await Proof._create("vcx_proof_create_with_proposal",
                                   constructor_params,
                                   c_params)

    @staticmethod
    async def deserialize(data: dict):
        """
        Builds a Proof object with defined attributes.
        Attributes are provided by a previous call to the serialize function.
        :param data:
        Example:
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        proof = await Proof.create(source_id, name, requested_attrs)
        data = proof.serialize()
        proof2 = await Proof.deserialize(data)
        :return: Proof Object
        """
        return await Proof._deserialize("vcx_proof_deserialize",
                                        json.dumps(data),
                                        data.get('data').get('source_id'))

    async def serialize(self) -> dict:
        """
        Data returned can be used to recreate an entity by passing it to the deserialize function.
        Same json object structure that is passed to the deserialize function.
        Example:
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        proof = await Proof.create(source_id, name, requested_attrs)
        data = proof.serialize()
        :return: String
        """
        return await self._serialize(Proof, 'vcx_proof_serialize')

    async def update_state(self) -> int:
        """
        Query the agency for the received messages.
        Checks for any messages changing state in the object and updates the state attribute.
        Example:
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        proof = await Proof.create(source_id, name, requested_attrs)
        assert await proof.update_state() == State.Initialized
        :return: StateType
        """
        return await self._update_state(Proof, 'vcx_proof_update_state')

    async def update_state_with_message(self, message: str) -> int:
        """
        Update the state of the proof based on the given message.
        Example:
        proof = await Proof.create(source_id)
        assert await proof.update_state_with_message(message) == State.Accepted
        :param message: message to process for state changes
        :return Current state of the Proof
        """
        return await self._update_state_with_message(Connection, message, 'vcx_proof_update_state_with_message')

    async def get_state(self) -> int:
        """
        Gets the state of the entity.
        Example:
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        proof = await Proof.create(source_id, name, requested_attrs)
        assert await proof.get_state() == State.Initialized
        :return: Possible states:
                  1 - Initialized
                  2 - Request Sent
                  3 - Proof Received
                  4 - Accepted
        """
        return await self._get_state(Proof, 'vcx_proof_get_state')

    def release(self) -> None:
        """
        Internal method used for memory management
        :return: None
        """
        self._release(Proof, 'vcx_proof_release')

    async def get_proof_request_msg(self):
        """
        Gets the proof request message that can be sent to the specified connection
        Example:
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        proof = await Proof.create(source_id, name, requested_attrs)
        await proof.get_proof_request_msg()
        :param
        :return: proof request message
            {'@topic': {'tid': 0, 'mid': 0}, '@type': {'version': '1.0', 'name': 'PROOF_REQUEST'}, 'proof_request_data': {'name': 'proof_req', 'nonce': '118065925949165739229152', 'version': '0.1', 'requested_predicates': {}, 'non_revoked': None, 'requested_attributes': {'attribute_0': {'name': 'name', 'restrictions': {'$or': [{'issuer_did': 'did'}]}}}, 'ver': '1.0'}, 'thread_id': '40bdb5b2'}
        """
        if not hasattr(Proof.get_proof_request_msg, "cb"):
            self.logger.debug("vcx_proof_get_request_msg: Creating callback")
            Proof.get_proof_request_msg.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_proof_handle = c_uint32(self.handle)

        msg = await do_call('vcx_proof_get_request_msg',
                            c_proof_handle,
                            Proof.get_proof_request_msg.cb)

        return json.loads(msg.decode())

    async def get_proof_request_attach(self):
        """
        Gets the proof request attachment for ephemeral proof request
        Example:
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        proof = await Proof.create(source_id, name, requested_attrs)
        await proof.get_proof_request_attach()
        :param
        :return: proof request attachment
        {"@id": "8b23c2b6-b432-45d8-a377-d003950c0fcc", "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/request-presentation", "comment": "Person Proving", "request_presentations~attach": [{"@id": "libindy-request-presentation-0", "data": {"base64": "eyJuYW1lIjoiUGVyc29uIFByb3ZpbmciLCJub25fcmV2b2tlZCI6bnVsbCwibm9uY2UiOiI2MzQxNzYyOTk0NjI5NTQ5MzA4MjY1MzQiLCJyZXF1ZXN0ZWRfYXR0cmlidXRlcyI6eyJhdHRyaWJ1dGVfMCI6eyJuYW1lIjoibmFtZSJ9LCJhdHRyaWJ1dGVfMSI6eyJuYW1lIjoiZW1haWwifX0sInJlcXVlc3RlZF9wcmVkaWNhdGVzIjp7fSwidmVyIjpudWxsLCJ2ZXJzaW9uIjoiMS4wIn0="}, "mime-type": "application/json"}]}
        TODO: add attachment
        """
        if not hasattr(Proof.get_proof_request_attach, "cb"):
            self.logger.debug("vcx_proof_get_request_attach: Creating callback")
            Proof.get_proof_request_attach.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_proof_handle = c_uint32(self.handle)
        msg = await do_call('vcx_proof_get_request_attach',
                                          c_proof_handle,
                                          Proof.get_proof_request_attach.cb)

        return json.loads(msg.decode())

    async def get_proof_proposal(self):
        """
        Gets the proof proposal received.
        :return: proof proposal
        """
        if not hasattr(Proof.get_proof_proposal, "cb"):
            self.logger.debug("get_proof_proposal: Creating callback")
            Proof.get_proof_proposal.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_proof_handle = c_uint32(self.handle)

        msg = await do_call('vcx_get_proof_proposal',
                            c_proof_handle,
                            Proof.get_proof_proposal.cb)

        return json.loads(msg.decode())

    async def request_proof_presentation(self, name: str, connection: Connection, requested_attrs: list, revocation_interval: dict, requested_predicates=None):
        """
        Respond to proposal with different proof request (enables negotiation).

        :param name: Name of the Proof
        :param connection: Connection to send proof request
        :param requested_attrs: Attributes associated with the Proof
           {
               "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
               "names": Optional<[string, string]>, // attribute names, (case insensitive and ignore spaces)
                                                    // NOTE: should either be "name" or "names", not both and not none of them.
                                                    // Use "names" to specify several attributes that have to match a single credential.
               "restrictions":  (filter_json) {
                  "schema_id": string, (Optional)
                  "schema_issuer_did": string, (Optional)
                  "schema_name": string, (Optional)
                  "schema_version": string, (Optional)
                  "issuer_did": string, (Optional)
                  "cred_def_id": string, (Optional)
              },
               "non_revoked": {
                   "from": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
                   "to": Optional<(u64)>
                       //Requested time represented as a total number of seconds from Unix Epoch, Optional
               }
           }
        :param requested_predicates: Predicates associated with the Proof
           { // set of requested predicates
              "name": attribute name, (case insensitive and ignore spaces)
              "p_type": predicate type (">=", ">", "<=", "<")
              "p_value": int predicate value
              "restrictions": Optional<filter_json>, // see above
              "non_revoked": Optional<{
                  "from": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
                  "to": Optional<(u64)> Requested time represented as a total number of seconds from Unix Epoch, Optional
              }>
           }
        :param revocation_interval: interval applied to all requested attributes indicating when the claim must be valid (NOT revoked)
        :return:
        """
        if requested_predicates is None:
            requested_predicates = []

        if not hasattr(Proof.request_proof_presentation, "cb"):
            self.logger.debug("vcx_proof_request_proof: Creating callback")
            Proof.request_proof_presentation.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_proof_handle = c_uint32(self.handle)
        c_connection_handle = c_uint32(connection.handle)
        c_name = c_char_p(name.encode('utf-8'))
        c_req_attrs = c_char_p(json.dumps(requested_attrs).encode('utf-8'))
        c_req_predicates = c_char_p(json.dumps(requested_predicates).encode('utf-8'))
        c_revocation_interval = c_char_p(json.dumps(revocation_interval).encode('utf-8'))

        await do_call('vcx_proof_request_proof',
                      c_proof_handle,
                      c_connection_handle,
                      c_req_attrs,
                      c_req_predicates,
                      c_revocation_interval,
                      c_name,
                      Proof.request_proof_presentation.cb)

    async def request_proof(self, connection: Connection):
        """
        Sends a proof request message to the specified connection
        Example:
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        proof = await Proof.create(source_id, name, requested_attrs)
        await proof.request_proof(connection)
        :param connection: Connection to send proof request
        :return:
        """
        if not hasattr(Proof.request_proof, "cb"):
            self.logger.debug("vcx_proof_send_request: Creating callback")
            Proof.request_proof.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_proof_handle = c_uint32(self.handle)
        c_connection_handle = c_uint32(connection.handle)

        await do_call('vcx_proof_send_request',
                      c_proof_handle,
                      c_connection_handle,
                      Proof.request_proof.cb)

    async def get_proof(self, connection: Connection) -> list:
        """
        Todo: This should be depricated, use get_proof_msg
        Gets the proof message
        Example:
        connection = await Connection.create(source_id)
        await connection.connect(phone_number)
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        proof = await Proof.create(source_id, name, requested_attrs)
        await proof.request_proof(connection)
        await proof.get_proof(connection)
        :param connection: Handle for the connection to receive a proof from.
        :return: List of proofs received from the given connection.
        """
        if not hasattr(Proof.get_proof, "cb"):
            self.logger.debug("vcx_get_proof: Creating callback")
            Proof.get_proof.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_uint32, c_char_p))

        c_proof_handle = c_uint32(self.handle)
        c_connection_handle = c_uint32(connection.handle)

        proof_state, proof = await do_call('vcx_get_proof',
                                           c_proof_handle,
                                           c_connection_handle,
                                           Proof.get_proof.cb)
        self.proof_state = proof_state
        return json.loads(proof.decode())

    async def get_proof_msg(self) -> list:
        """
        Example:
        name = "proof name"
        requested_attrs = [{"name": "age", "restrictions": [{"schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766" } ] }, { "name":"name", "restrictions": [ { "schema_id": "6XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"Faber Student Info", "schema_version":"1.0", "schema_issuer_did":"6XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"8XFh8yBzrpJQmNyZzgoTqB", "cred_def_id": "8XFh8yBzrpJQmNyZzgoTqB:3:CL:1766" }, { "schema_id": "5XFh8yBzrpJQmNyZzgoTqB:2:schema_name:0.0.11", "schema_name":"BYU Student Info", "schema_version":"1.0", "schema_issuer_did":"5XFh8yBzrpJQmNyZzgoTqB", "issuer_did":"66Fh8yBzrpJQmNyZzgoTqB", "cred_def_id": "66Fh8yBzrpJQmNyZzgoTqB:3:CL:1766"}]}]
        proof = await Proof.create(source_id, name, requested_attrs)
        await proof.request_proof()
        await proof.get_proof_msg()
        :return: List of proofs received for this specific proof object
        """
        if not hasattr(Proof.get_proof, "cb"):
            self.logger.debug("vcx_get_proof: Creating callback")
            Proof.get_proof_msg.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_uint32, c_char_p))

        c_proof_handle = c_uint32(self.handle)

        proof_state, proof = await do_call('vcx_get_proof_msg',
                                           c_proof_handle,
                                           Proof.get_proof_msg.cb)
        self.proof_state = proof_state
        return json.loads(proof.decode())

    async def set_connection(self, connection: Connection):
        """
        Set connection for created proof. Used for Out-Of-Band with presentation request attachment
        """
        if not hasattr(Proof.set_connection, "cb"):
            self.logger.debug("vcx_proof_set_connection: Creating callback")
            Proof.set_connection.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32))

        c_proof_handle = c_uint32(self.handle)
        c_connection_handle = c_uint32(connection.handle)

        await do_call('vcx_proof_set_connection',
                      c_proof_handle,
                      c_connection_handle,
                      Proof.set_connection.cb)

    async def get_problem_report(self) -> Optional[str]:
        """
        Get Problem Report message for object in Failed or Rejected state.
        :return: Problem Report as JSON string or null
        """

        if not hasattr(Proof.get_problem_report, "cb"):
            self.logger.debug("vcx_proof_get_problem_report: Creating callback")
            Proof.get_problem_report.cb = create_cb(CFUNCTYPE(None, c_uint32, c_uint32, c_char_p))

        c_connection_handle = c_uint32(self.handle)
        result = await do_call('vcx_proof_get_problem_report',
                               c_connection_handle,
                               Proof.get_problem_report.cb)

        self.logger.debug("vcx_proof_get_problem_report completed")
        return result.decode() if result else None
