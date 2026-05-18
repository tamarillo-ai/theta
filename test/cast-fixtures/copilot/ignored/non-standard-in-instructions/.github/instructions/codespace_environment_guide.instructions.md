---
applyTo: "**"
---

# Codespace 개발 환경 설정 가이드

이 문서는 `github_lecture` 또는 복제한 실습 리포지토리에서 **GitHub Codespaces**를 통해 Copilot agent mode 실습 환경을 마련하는 방법을 안내합니다.

## 1. 목적
- 브라우저/VS Code에서 즉시 사용할 수 있는 Codespace를 만들어, Python/Django 백엔드와 React 프론트엔드를 동시에 실행할 수 있도록 준비합니다.
- Copilot agent mode가 곧바로 Actions/Devcontainer와 연결되어 실습 흐름(브랜치 생성 → Copilot 작업 → PR 준비)을 끊김 없이 이어가도록 합니다.

## 2. 기본 구성
- `.devcontainer/devcontainer.json`을 생성하여 Codespace가 로드될 때 필요한 도구(예: Python 3.11, Node 18, pipenv 등)를 자동 설치합니다.
- `devcontainer.json` 안에 `forwardPorts` 목록으로 `8000`, `3000`, `27017`을 포함시켜, backend/frontend/db에 대한 포트를 미리 개방합니다.
- `postCreateCommand`/`postStartCommand`로 `pip install -r backend/requirements.txt`와 `npm install frontend`를 실행하여 의존성을 설치합니다.

## 3. Codespace 생성/진입 절차
1. 리포지토리 페이지에서 **Create Codespace** 버튼을 눌러 실습 복사본을 선택한 뒤 Codespace를 만듭니다.
2. Codespace가 로드되면 터미널에서 `pip install -r backend/requirements.txt`와 `npm install frontend`를 다시 확인합니다(필요 시 `npm run build`도 실행).
3. `.devcontainer/`에서 환경 변수(template.env 등)를 `settings.env`로 복사하거나 Codespace `Settings` > `Secrets`에 필요한 값을 추가합니다.
4. `python manage.py migrate` → `python manage.py runserver 0.0.0.0:8000` + `npm start --prefix frontend -- --host 0.0.0.0 --port 3000` 순으로 backend/frontend를 실행합니다.
5. Copilot Chat(우측 상단 Copilot 아이콘)과 terminal, port panel(Ctrl+J)을 확인하여 서비스가 정상적임을 확인합니다.

## 4. Agent mode 준비 팁
- Agent mode 커맨드를 실행할 때는 절대 디렉터리를 변경하지 말고, 명령에 `backend/` 또는 `frontend/` 경로를 포함시켜 실행합니다(예: `python backend/manage.py migrate`).
- Codespace가 생성된 즉시 GitHub Copilot Chat을 열고 GPT-5.3(or 최신)의 Agent 모드를 선택합니다. 필요 시 `Model switcher`에서 `gpt-5.3-codex`로 설정.
- Codespace 내 `.github/steps/1-preparing.md` 템플릿에 있는 prompt를 그대로 복사하여 사용하면 `build-octofit-app` 브랜치 생성/푸시 자동화에 도움이 됩니다.

## 5. 포트/보안
- 공개 포트: `8000`(Django), `3000`(React UI)
- 프라이빗 포트: `27017`(MongoDB). 외부 공개 금지.
- Codespaces Port Forwarding 탭에서 불필요한 포트를 제거하고, 공개로 변경해야 할 경우에는 반드시 인프라팀에게 승인 요청하세요.

## 6. 참고 자료
- `.github/prompts/github_copilot_training.prompt.md`
- `docs/octofit_story.md` (배경 및 시나리오)
- GitHub Docs: [GitHub Codespaces 개요](https://docs.github.com/en/codespaces/overview)
