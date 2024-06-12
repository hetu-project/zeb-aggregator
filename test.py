import requests


if __name__ == '__main__':

    base_url = "http://52.221.181.98:8080"

    url1 = base_url + "/gateway/overview"

    try:

        response1 = requests.get(url1)
        response1.raise_for_status()

        data = response1.json()

        if 'nodes' in data:
            for node in data['nodes']:

                item_id = node['node_id']
                # print(f"Received ID: {item_id}")

                url2 = f"{base_url}/gateway/node/{item_id}"

                response2 = requests.get(url2)
                response2.raise_for_status()

                detailed_data = response2.json()
                if len(detailed_data['message_list']) > 0:
                    print(f"Node id: {item_id}")
                    print(f"Node info: {detailed_data}")
                    for message in detailed_data['message_list']:
                        url3 = f"{base_url}/gateway/message/{message['message_id']}"
                        response3 = requests.get(url3)
                        response3.raise_for_status()
                        detailed_data = response3.json()
                        print(f"Message: {detailed_data}")


                        url4 = f"{base_url}/gateway/merge_log_by_message_id/{message['message_id']}"
                        response4 = requests.get(url4)
                        response4.raise_for_status()
                        detailed_data = response4.json()
                        print(f"merge log: {detailed_data}")

        else:
            print("nodes not found in the response data")

    except requests.exceptions.RequestException as e:
        print(f"Request failed: {e}")
