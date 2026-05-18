/**
 * 全局常量配置文件
 */

export const UPDATE_CONFIG = {
  GITHUB_OWNER: "b-yp",
  GITHUB_REPO: "baize",

  /**
   * 拼接 GitHub Latest Release API 链接
   */
  get LATEST_RELEASE_URL() {
    return `https://api.github.com/repos/${this.GITHUB_OWNER}/${this.GITHUB_REPO}/releases/latest`;
  },
};
