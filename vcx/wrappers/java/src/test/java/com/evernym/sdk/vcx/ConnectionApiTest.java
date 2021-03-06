package com.evernym.sdk.vcx;


import com.evernym.sdk.vcx.connection.AcceptConnectionResult;
import com.evernym.sdk.vcx.connection.ConnectionApi;
import com.evernym.sdk.vcx.vcx.VcxApi;
import com.evernym.sdk.vcx.utils.UtilsApi;
import java9.util.concurrent.CompletableFuture;
import org.awaitility.Awaitility;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.concurrent.ExecutionException;

import static com.evernym.sdk.vcx.TestHelper._createConnection;
import static com.evernym.sdk.vcx.TestHelper._createConnectionWithInvite;
import static org.junit.jupiter.api.Assertions.assertNotSame;

class ConnectionApiTest {

	String inviteDetails = "{'statusCode':'MS-101','connReqId':'NjcwOWU','senderDetail':{'name':'ent-name','agentKeyDlgProof':{'agentDID':'U5LXs4U7P9msh647kToezy','agentDelegatedKey':'FktSZg8idAVzyQZrdUppK6FTrfAzW3wWVzAjJAfdUvJq','signature':'gkVDhwe2/FEtFqJYBm2wbEvqGlBwAGGaC19Oebj/3ZtZ/KpZs7K2JFMgTqTb29xTTAad04AjfNa76931eRa6BA=='},'DID':'WRUzXXuFVTYkT8CjSZpFvT','logoUrl':'ent-logo-url','verKey':'ESE6MnqAyjRigduPG454vfLvKhMbmaZjy9vqxCnSKQnp'},'senderAgencyDetail':{'DID':'BDSmVkzxRYGE4HKyMKxd1H','verKey':'HsaWDKnJtgoBsyqG2zKa5xRvKZzZHhkiCDH7eU3iqRsv','endpoint':'localhost:9001/agency/msg'},'targetName':'there','statusMsg':'message created'}";
	String outofbandInvite= "{'@type':'https://didcomm.org/out-of-band/%VER/invitation','@id':'<idusedforcontextaspthid>','label':'FaberCollege','handshake_protocols':['https://didcomm.org/connections/1.0'],'service':[{'id':'#inline','type':'did-communication','recipientKeys':['did:key:z2DaR6DBCHc1bopEZ3D33P4arAm8vBeKvd1MrKw6Z7XQHSH'],'routingKeys':[],'serviceEndpoint':'https://example.com:5000'}]}";

	@BeforeEach
	void setup() throws Exception {
		System.setProperty(org.slf4j.impl.SimpleLogger.DEFAULT_LOG_LEVEL_KEY, "DEBUG");
		if (! TestHelper.vcxInitialized) {
			TestHelper.getResultFromFuture(VcxApi.vcxInit(TestHelper.VCX_CONFIG_TEST_MODE));
			TestHelper.vcxInitialized = true;
		}
	}


	@Test
	@DisplayName("create a connection")
	void createConnection() throws VcxException {

		long connectionHandle = _createConnection();
		assertNotSame(null, connectionHandle);
		assertNotSame(0, connectionHandle);
	}

	@Test
	@DisplayName("connect connection without phone number")
	void connectConnectionWithoutPhone() throws VcxException {
		String payload = "{ 'connection_type': 'SMS' }";
		Integer connectionHandle = _createConnection();
		CompletableFuture<String> future = ConnectionApi.vcxConnectionConnect(connectionHandle, TestHelper.convertToValidJson(payload));
		Awaitility.await().until(future::isDone);
		assertNotSame("", future.getNow(""));
	}

	@Test
	@DisplayName("connect connection with phone number")
	void connectConnectionWithPhone() throws VcxException {
		String payload = "{ 'connection_type': 'SMS', 'phone':'7202200000' }";
		Integer connectionHandle = _createConnection();
		CompletableFuture<String> future = ConnectionApi.vcxConnectionConnect(connectionHandle, TestHelper.convertToValidJson(payload));
		Awaitility.await().until(future::isDone);
		assertNotSame("", future.getNow(""));


	}

	@Test
	@DisplayName("throw invalid connection handle exception for wrong handle")
	void throwInvalidConnectionHandleException() {

		Assertions.assertThrows(ExecutionException.class, () -> {
			String payload = "{ 'connection_type': 'SMS', 'phone':'7202200000' }";
			TestHelper.getResultFromFuture(ConnectionApi.vcxConnectionConnect(8765, TestHelper.convertToValidJson(payload)));
		});


	}

	@Test
	@DisplayName("serialize a connection")
	void serializeConnection() throws VcxException {
		Integer connectionHandle = _createConnection();
		CompletableFuture<String> future = ConnectionApi.connectionSerialize(connectionHandle);
		Awaitility.await().until(future::isDone);
		String serializedJson = future.getNow("");
		System.out.println(serializedJson);
		assertNotSame("", serializedJson);
		assert (serializedJson.contains("version"));
		assert (serializedJson.contains("data"));
	}

	@Test
	@DisplayName("throw invalid connection handle exception for serializing invalid connection ")
	void serializeConnectionWithBadHandle() {
		Assertions.assertThrows(ExecutionException.class, () -> {
			TestHelper.getResultFromFuture(ConnectionApi.connectionSerialize(0));
		});
	}

	@Test
	@DisplayName("delete a connection")
	void deleteConnection() throws VcxException, ExecutionException, InterruptedException {
		Integer connectionHandle = _createConnection();
		String payload = "{ 'connection_type': 'SMS', 'phone':'7202200000' }";
		TestHelper.getResultFromFuture(ConnectionApi.vcxConnectionConnect(connectionHandle, TestHelper.convertToValidJson(payload)));
		CompletableFuture<Void> futureDelete = ConnectionApi.deleteConnection(connectionHandle);
		Awaitility.await().until(futureDelete::isDone);
	}

	@Test
	@DisplayName("throw invalid connection handle exception if trying to serialize deleted connection ")
	void serlializeDeletedConnection() {

		Assertions.assertThrows(ExecutionException.class, () -> {
			Integer connectionHandle = _createConnection();
			TestHelper.getResultFromFuture(ConnectionApi.deleteConnection(connectionHandle));
			TestHelper.getResultFromFuture(ConnectionApi.connectionSerialize(connectionHandle));
		});

	}

	@Test
	@DisplayName("throw invalid connection handle exception if trying to serialize released connection")
	void serlializeReleasedConnection() {
		Assertions.assertThrows(ExecutionException.class, () -> {
			Integer connectionHandle = _createConnection();
			int releaseResult = ConnectionApi.connectionRelease(connectionHandle);
			assert (releaseResult == 0);
			TestHelper.getResultFromFuture(ConnectionApi.connectionSerialize(connectionHandle));
		});
	}

	@Test
	@DisplayName("release a connection")
	void releaseConnection() throws VcxException {
		Integer connectionHandle = _createConnection();
		int result = ConnectionApi.connectionRelease(connectionHandle);
		assert (result == 0);
	}

	@Test
	@DisplayName("initialise a connection")
	void initialiseConnection() throws VcxException, ExecutionException, InterruptedException {
		Integer connectionHandle = _createConnection();
		CompletableFuture<Integer> futureUpdateState = ConnectionApi.vcxConnectionUpdateState(connectionHandle);
		Awaitility.await().until(futureUpdateState::isDone);
		int updateStateResult = futureUpdateState.get();
		assert (updateStateResult == 1);
		CompletableFuture<Integer> futureGetState = ConnectionApi.connectionGetState(connectionHandle);
		Awaitility.await().until(futureGetState::isDone);
		assert (futureGetState.get() == updateStateResult);

	}

	@Test
	@DisplayName("send offer connection")
	void sendOfferConnection() throws VcxException, ExecutionException, InterruptedException {
		String payload = "{ 'connection_type': 'SMS', 'phone':'7202200000' }";
		Integer connectionHandle = _createConnection();
		CompletableFuture<String> future = ConnectionApi.vcxConnectionConnect(connectionHandle, TestHelper.convertToValidJson(payload));
		Awaitility.await().until(future::isDone);
		CompletableFuture<Integer> futureGetState = ConnectionApi.connectionGetState(connectionHandle);
		Awaitility.await().until(futureGetState::isDone);
		int connectionState = futureGetState.get();
		assert (connectionState == 2);
		UtilsApi.vcxMockSetAgencyResponse(9);
		CompletableFuture<Integer> futureUpdateState = ConnectionApi.vcxConnectionUpdateState(connectionHandle);
		Awaitility.await().until(futureUpdateState::isDone);
		int updateStateResult = futureUpdateState.get();
		assert (updateStateResult == 4);
	}

	@Test
	@DisplayName("get abbreviated invite detials")
	void inviteDetailsAbbreviatedConnection() throws VcxException, ExecutionException, InterruptedException {
		String payload = "{ 'connection_type': 'SMS', 'phone':'7202200000' }";
		int connectionHandle = _createConnection();
		CompletableFuture<String> acceptInvitation = ConnectionApi.vcxConnectionConnect(connectionHandle, TestHelper.convertToValidJson(payload));
		Awaitility.await().until(acceptInvitation::isDone);
		CompletableFuture<String> detials = ConnectionApi.connectionInviteDetails(connectionHandle, 1);
		Awaitility.await().until(detials::isDone);
		assert (detials.get().contains("dp"));

	}

	@Test
	@DisplayName("get un-abbreviated invite detials")
	void inviteDetailsUnAbbreviatedConnection() throws VcxException, ExecutionException, InterruptedException {
		String payload = "{ 'connection_type': 'SMS', 'phone':'7202200000' }";
		int connectionHandle = _createConnection();
		CompletableFuture<String> acceptInvitation = ConnectionApi.vcxConnectionConnect(connectionHandle, TestHelper.convertToValidJson(payload));
		Awaitility.await().until(acceptInvitation::isDone);
		CompletableFuture<String> detials = ConnectionApi.connectionInviteDetails(connectionHandle, 0);
		Awaitility.await().until(detials::isDone);
		assert (detials.get().contains("senderAgencyDetail"));

	}

	@Test
	@DisplayName("test update_state_with_message")
	void updateStateWithMessage() throws VcxException, ExecutionException, InterruptedException {
		String payload = "{ 'connection_type': 'QR' }";
		int connectionHandle = _createConnection();
		CompletableFuture<String> acceptInvitation = ConnectionApi.vcxConnectionConnect(connectionHandle, TestHelper.convertToValidJson(payload));
		Awaitility.await().until(acceptInvitation::isDone);
		String message = "{ \"statusCode\": \"MS-104\", \"payload\": [ -126, -91, 64, 116, 121, 112, 101, -125, -92, 110, 97, 109, 101, -83, 67, 111, 110, 110, 82, 101, 113, 65, 110, 115, 119, 101, 114, -93, 118, 101, 114, -93, 49, 46, 48, -93, 102, 109, 116, -92, 106, 115, 111, 110, -92, 64, 109, 115, 103, -36, 1, 79, -48, -127, -48, -84, 115, 101, 110, 100, 101, 114, 68, 101, 116, 97, 105, 108, -48, -122, -48, -93, 68, 73, 68, -48, -74, 75, 119, 49, 57, 54, 87, 75, 69, 72, 77, 98, 85, 105, 86, 71, 99, 85, 76, 120, 56, 107, 82, -48, -80, 97, 103, 101, 110, 116, 75, 101, 121, 68, 108, 103, 80, 114, 111, 111, 102, -48, -125, -48, -88, 97, 103, 101, 110, 116, 68, 73, 68, -48, -74, 76, 115, 102, 102, 106, 72, 114, 69, 52, 86, 113, 75, 66, 114, 69, 69, 99, 99, 86, 89, 75, 86, -48, -79, 97, 103, 101, 110, 116, 68, 101, 108, 101, 103, 97, 116, 101, 100, 75, 101, 121, -48, -39, 44, 66, 113, 70, 52, 113, 119, 85, 97, 104, 68, 109, 114, 116, 115, 113, 54, 111, 88, 109, 77, 112, 116, 89, 105, 57, 76, 109, 109, 70, 121, 102, 57, 55, 76, 111, 103, 53, 75, 86, 69, 83, 98, 121, 105, -48, -87, 115, 105, 103, 110, 97, 116, 117, 114, 101, -48, -39, 88, 67, 76, 57, 90, 65, 113, 119, 72, 82, 54, 70, 110, 112, 106, 118, 49, 106, 80, 47, 115, 121, 103, 65, 43, 74, 78, 57, 74, 104, 120, 69, 65, 68, 86, 117, 101, 71, 88, 83, 101, 90, 54, 73, 72, 75, 97, 43, 52, 106, 57, 105, 108, 82, 111, 74, 49, 119, 76, 56, 66, 121, 54, 119, 97, 117, 86, 56, 113, 72, 86, 71, 49, 71, 74, 112, 101, 49, 71, 79, 106, 67, 105, 108, 101, 65, 65, 61, 61, -48, -89, 108, 111, 103, 111, 85, 114, 108, -48, -64, -48, -92, 110, 97, 109, 101, -48, -64, -48, -87, 112, 117, 98, 108, 105, 99, 68, 73, 68, -48, -64, -48, -90, 118, 101, 114, 75, 101, 121, -48, -39, 44, 66, 75, 84, 50, 67, 85, 78, 71, 66, 82, 107, 81, 67, 104, 54, 118, 85, 89, 118, 65, 111, 110, 101, 107, 110, 54, 88, 75, 122, 122, 122, 86, 68, 90, 107, 98, 114, 74, 85, 56, 86, 104, 99, 114 ], \"senderDID\": \"NsQ1rvm6TrsHx1TB4xEh55\", \"uid\": \"owm5yta\", \"type\": \"connReqAnswer\", \"deliveryDetails\": [] }";
		CompletableFuture<Integer> futureUpdateState = ConnectionApi.vcxConnectionUpdateStateWithMessage(connectionHandle, message);
		Awaitility.await().until(futureUpdateState::isDone);
		int updateStateResult = futureUpdateState.get();
		assert (updateStateResult == 4);
	}

	@Test
	@DisplayName("send ping")
	void sendPing() {
		Assertions.assertThrows(ExecutionException.class, () -> {
			Integer connectionHandle = _createConnection();
			TestHelper.getResultFromFuture(ConnectionApi.connectionSendPing(connectionHandle, null));
		});
	}

	@Test
	@DisplayName("send discovery features")
	void sendDiscoveryFeatures() {
		Assertions.assertThrows(ExecutionException.class, () -> {
			Integer connectionHandle = _createConnection();
			TestHelper.getResultFromFuture(ConnectionApi.connectionSendDiscoveryFeatures(connectionHandle, null, null));
		});
	}

    @Test
    @DisplayName("test redirect")
    void redirectConnection() throws VcxException, ExecutionException, InterruptedException {
        int redirectConnectionHandle = _createConnection();
        int connectionHandle = _createConnectionWithInvite(TestHelper.convertToValidJson(inviteDetails));
        CompletableFuture<Void> redirectInvitation = ConnectionApi.vcxConnectionRedirect(connectionHandle, redirectConnectionHandle);
        Awaitility.await().until(redirectInvitation::isDone);
        CompletableFuture<Integer> futureGetState = ConnectionApi.connectionGetState(connectionHandle);
        Awaitility.await().until(futureGetState::isDone);
        int state = futureGetState.get();
        assert(state == 8);
    }

    @Test
    @DisplayName("test update state with redirection msg and get redirect details")
    void updateStateWithRedirectDetails() throws VcxException, ExecutionException, InterruptedException {
        String payload= "{ 'connection_type': 'QR' }";
        int connectionHandle = _createConnection();
        CompletableFuture<String> acceptInvitation = ConnectionApi.vcxConnectionConnect(connectionHandle, TestHelper.convertToValidJson(payload));
        Awaitility.await().until(acceptInvitation::isDone);
        String message = "{ \"statusCode\": \"MS-107\", \"payload\": [-110, -109, -81, 99, 111, 110, 110, 82, 101, 113, 82, 101, 100, 105, 114, 101, 99, 116, -93, 49, 46, 48, -84, 105, 110, 100, 121, 46, 109, 115, 103, 112, 97, 99, 107, -36, 0, -24, -48, -111, -48, -105, -48, -74, 57, 54, 106, 111, 119, 113, 111, 84, 68, 68, 104, 87, 102, 81, 100, 105, 72, 49, 117, 83, 109, 77, -48, -39, 44, 66, 105, 118, 78, 52, 116, 114, 53, 78, 88, 107, 69, 103, 119, 66, 56, 81, 115, 66, 51, 109, 109, 109, 122, 118, 53, 102, 119, 122, 54, 85, 121, 53, 121, 112, 122, 90, 77, 102, 115, 74, 56, 68, 122, -48, -64, -48, -74, 56, 88, 70, 104, 56, 121, 66, 122, 114, 112, 74, 81, 109, 78, 121, 90, 122, 103, 111, 84, 113, 66, -48, -39, 44, 69, 107, 86, 84, 97, 55, 83, 67, 74, 53, 83, 110, 116, 112, 89, 121, 88, 55, 67, 83, 98, 50, 112, 99, 66, 104, 105, 86, 71, 84, 57, 107, 87, 83, 97, 103, 65, 56, 97, 57, 84, 54, 57, 65, -48, -64, -48, -39, 88, 77, 100, 115, 99, 66, 85, 47, 99, 89, 75, 72, 49, 113, 69, 82, 66, 56, 80, 74, 65, 43, 48, 51, 112, 121, 65, 80, 65, 102, 84, 113, 73, 80, 74, 102, 52, 84, 120, 102, 83, 98, 115, 110, 81, 86, 66, 68, 84, 115, 67, 100, 119, 122, 75, 114, 52, 54, 120, 87, 116, 80, 43, 78, 65, 68, 73, 57, 88, 68, 71, 55, 50, 50, 103, 113, 86, 80, 77, 104, 117, 76, 90, 103, 89, 67, 103, 61, 61], \"senderDID\": \"NsQ1rvm6TrsHx1TB4xEh55\", \"uid\": \"owm5yta\", \"type\": \"connReqRedirect\", \"deliveryDetails\": [] }";
        CompletableFuture<Integer> futureUpdateState = ConnectionApi.vcxConnectionUpdateStateWithMessage(connectionHandle, message);
        Awaitility.await().until(futureUpdateState::isDone);
        int updateStateResult = futureUpdateState.get();
        assert(updateStateResult == 8);

        CompletableFuture<String> featureGetRedirectDetails = ConnectionApi.vcxConnectionGetRedirectDetails(connectionHandle);
        Awaitility.await().until(featureGetRedirectDetails::isDone);
        assert(featureGetRedirectDetails.get().contains("DID"));
    }

	@Test
	@DisplayName("connection info")
	void connectionInfo() {
		Assertions.assertThrows(ExecutionException.class, () -> {
			Integer connectionHandle = _createConnection();
			TestHelper.getResultFromFuture(ConnectionApi.connectionInfo(connectionHandle));
		});
	}

	@Test
	@DisplayName("accept connection invitation")
	void acceptConnectionInvitation() throws VcxException, ExecutionException, InterruptedException {
		String connectionId = "testConnectionId";
		String payload = "{ 'connection_type': 'QR' }";

		AcceptConnectionResult result =
				TestHelper.getResultFromFuture(
						ConnectionApi.vcxConnectionAcceptConnectionInvite(connectionId,
								TestHelper.convertToValidJson(inviteDetails),
								TestHelper.convertToValidJson(payload)));
		assertNotSame(null, result.getConnectionHandle());
		assertNotSame(0, result.getConnectionSerialized());
	}

	@Test
	@DisplayName("create connection with out-of-band invitation")
	void createConnectionWithOutofbandInvitation() throws VcxException, ExecutionException, InterruptedException {
		int connection = TestHelper.getResultFromFuture(
				ConnectionApi.vcxCreateConnectionWithOutofbandInvite("test", TestHelper.convertToValidJson(outofbandInvite)));
		assertNotSame(0, connection);
	}

	@Test
	@DisplayName("send reuse")
	void sendReuse() {
		Assertions.assertThrows(ExecutionException.class, () -> {
			int connectionHandle = _createConnection();
			TestHelper.getResultFromFuture(ConnectionApi.connectionSendReuse(connectionHandle, TestHelper.convertToValidJson(outofbandInvite)));
		});
	}

	@Test
	@DisplayName("connection create outofband")
	void connectionCreateOutofband() {
		Assertions.assertThrows(ExecutionException.class, () -> {
			TestHelper.getResultFromFuture(
					ConnectionApi.vcxConnectionCreateOutofband("Foo", null, "Foo Goal", true, null));
		});
	}

	@Test
	@DisplayName("send answer")
	void sendAnswer() {
		Assertions.assertThrows(ExecutionException.class, () -> {
			int connectionHandle = _createConnection();
			String question = "{\"@type\": \"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/questionanswer/1.0/question\",\"@id\": \"518be002-de8e-456e-b3d5-8fe472477a86\",\"question_text\": \"Alice, are you on the phone with Bob from Faber Bank right now?\",}";
			String answer = "{\"text\": \"Yes, it's me\"}";
			TestHelper.getResultFromFuture(ConnectionApi.connectionSendAnswer(connectionHandle, question, answer));
		});
	}

	@Test
	@DisplayName("send invite action")
	void sendInviteAction() {
		Assertions.assertThrows(ExecutionException.class, () -> {
			int connectionHandle = _createConnection();
			String data = "{\"goal_code\": \"automotive.inspect.tire\"}";
			TestHelper.getResultFromFuture(ConnectionApi.connectionSendInviteAction(connectionHandle, data));
		});
	}
}
