import requests
import random
import string
import concurrent.futures

API_URL = "http://localhost:3000/posts"

# 랜덤 문자열 생성 함수
def random_string(length=10):
    return ''.join(random.choices(string.ascii_letters + string.digits, k=length))

# 단일 POST 요청 함수
def create_post(_):
    payload = {
        "title": random_string(20),
        "content": random_string(200),
        "user_id": 1  # user_id는 1~10 랜덤
    }
    try:
        response = requests.post(API_URL, json=payload)
        return response.status_code
    except Exception as e:
        return f"Error: {e}"

# 10000개 요청 병렬 처리
def main():
    total_posts = 10000
    max_workers = 50  # 동시에 실행할 스레드 수

    with concurrent.futures.ThreadPoolExecutor(max_workers=max_workers) as executor:
        results = list(executor.map(create_post, range(total_posts)))

    print("완료된 요청 수:", len(results))
    print("상태코드 예시:", results[:10])

if __name__ == "__main__":
    main()
