// A ajuda do 7Hinos: artigos curtos de "como fazer", escritos para quem tem
// pouca prática com computador. Cada um leva à tela certa ("Me leve lá").
// É também a base da busca "Pergunte ao 7Hinos" (ver search.ts) e, no futuro,
// da wiki.

import type { Route } from "../state/nav.svelte";

export interface Article {
  id: string;
  title: string;
  /** Uma frase: o que o artigo resolve. */
  summary: string;
  steps: string[];
  /** Jeitos de perguntar que não aparecem no texto ("tv", "datashow"...). */
  keywords: string[];
  category: Category;
  /** A tela onde se faz isso. */
  route?: Route;
}

export type Category = "Começando" | "Hinos" | "Telas" | "Culto" | "Bíblia e apresentações" | "Avisos" | "Problemas";

export const CATEGORIES: Category[] = ["Começando", "Hinos", "Telas", "Culto", "Bíblia e apresentações", "Avisos", "Problemas"];

export const ARTICLES: Article[] = [
  // Começando
  {
    id: "tocar-hino",
    title: "Tocar um hino e mostrar a letra no telão",
    summary: "Ache o hino pelo nome ou número e projete com um clique.",
    steps: [
      "Digite o nome, o número ou um pedaço da letra na busca (no alto, à esquerda).",
      "Clique no hino para abrir no painel da direita.",
      "Escolha Cantado (com voz), Playback (só a música) ou Só letra.",
      "Clique em Projetar. A letra vai para o telão e passa sozinha com a música.",
    ],
    keywords: ["cantar", "musica", "louvor", "cancao", "colocar", "passar", "slide", "comecar", "iniciar", "numero"],
    category: "Começando",
    route: { name: "hymnal" },
  },
  {
    id: "modo-simples",
    title: "Deixar o 7Hinos mais simples (botões grandes)",
    summary: "O modo Simples mostra só o essencial, com letras e botões maiores.",
    steps: [
      "Abra Ajustes (no canto de baixo, à esquerda).",
      "Em Modo de uso, escolha Simples.",
      "Para voltar, faça o mesmo e escolha Completo.",
    ],
    keywords: ["facil", "grande", "idoso", "letra grande", "botao grande", "simplificar", "aumentar"],
    category: "Começando",
    route: { name: "settings" },
  },
  {
    id: "tamanho-texto",
    title: "Aumentar o texto e os botões do programa",
    summary: "Deixe tudo maior na tela do computador (não muda o telão).",
    steps: ["Abra Ajustes.", "Em Aparência → Tamanho do texto e dos botões, escolha Grande ou Muito grande."],
    keywords: ["zoom", "letra pequena", "enxergar", "ver melhor", "aumentar fonte", "lupa"],
    category: "Começando",
    route: { name: "settings" },
  },
  {
    id: "tour",
    title: "Ver o passeio pelo programa de novo",
    summary: "Um passeio rápido mostrando onde fica cada coisa.",
    steps: ["Abra a Ajuda: na barra lateral ou, no modo simples, o botão Ajuda no alto. A tecla F1 também abre.", "Clique em Ver o passeio de novo."],
    keywords: ["tutorial", "tour", "apresentacao", "aprender", "primeira vez", "como usar", "guia"],
    category: "Começando",
    route: { name: "help" },
  },
  {
    id: "atualizar",
    title: "Atualizar o 7Hinos",
    summary: "O programa se atualiza sozinho; às vezes pede para reiniciar.",
    steps: [
      "Com internet, o 7Hinos procura versão nova ao abrir.",
      "Quando aparecer o aviso de versão nova, clique em Reiniciar agora.",
      "Para ver o que mudou, abra Ajustes → Sobre → Novidades.",
    ],
    keywords: ["versao nova", "update", "atualizacao", "novidades", "reiniciar"],
    category: "Começando",
    route: { name: "settings" },
  },

  // Hinos
  {
    id: "buscar-numero",
    title: "Achar um hino pelo número",
    summary: "Digite só o número do hinário e aperte Enter para tocar.",
    steps: [
      "Em qualquer tela, comece a digitar o número (por exemplo 18).",
      "A busca abre sozinha com o hino.",
      "Aperte Enter para tocar na hora, ou clique no hino para só abrir.",
    ],
    keywords: ["numero", "hinario", "achar", "procurar", "encontrar", "pesquisar"],
    category: "Hinos",
    route: { name: "hymnal" },
  },
  {
    id: "playback",
    title: "Tocar só a música, sem voz (playback)",
    summary: "Use o playback quando a igreja ou o coral vai cantar.",
    steps: ["Abra o hino.", "Escolha Playback antes de clicar em Projetar.", "Se o hino não tiver playback, o botão fica apagado."],
    keywords: ["instrumental", "sem voz", "karaoke", "acompanhamento", "fundo musical", "coral"],
    category: "Hinos",
  },
  {
    id: "so-letra",
    title: "Mostrar só a letra, sem tocar música",
    summary: "Para quando alguém vai tocar ao vivo (piano, violão, banda).",
    steps: ["Abra o hino.", "Escolha Só letra e clique em Projetar.", "Passe as estrofes com as setas do teclado ou com o passador."],
    keywords: ["sem musica", "sem audio", "ao vivo", "piano", "violao", "banda", "teclado", "mudo"],
    category: "Hinos",
  },
  {
    id: "passar-estrofe",
    title: "Passar para a próxima estrofe ou voltar",
    summary: "Setas do teclado, passador ou clique na estrofe.",
    steps: [
      "Seta para a direita ou para baixo: próxima. Para a esquerda ou para cima: anterior.",
      "Ou clique direto na estrofe, na lista do painel da direita.",
      "Passadores de slide (aquele controle de apresentação) também funcionam.",
    ],
    keywords: ["proximo", "voltar", "avancar", "seta", "passador", "controle", "slide", "estrofe", "verso", "refrao"],
    category: "Hinos",
  },
  {
    id: "tela-preta",
    title: "Deixar o telão preto por um momento",
    summary: "Esconde a letra sem parar a música.",
    steps: ["Aperte a tecla B (ou o ponto) no teclado.", "Ou clique no botão do sol e da lua, ao lado de Projetar.", "Aperte de novo para voltar."],
    keywords: ["apagar", "esconder", "escuro", "preto", "sumir", "blackout", "tirar"],
    category: "Hinos",
  },
  {
    id: "trocar-hino",
    title: "Trocar de hino na hora, com outro tocando",
    summary: "Clique duplo no hino da fila troca imediatamente.",
    steps: [
      "Com um hino tocando, dê clique duplo em outro: ele fica como Próximo.",
      "Dê clique duplo de novo nele (ou clique em Já) para trocar na hora.",
      "No Culto de hoje, clique duplo num hino já começa ele direto.",
    ],
    keywords: ["mudar", "outro hino", "fila", "proximo", "interromper", "parar e tocar"],
    category: "Hinos",
  },
  {
    id: "sem-internet",
    title: "Usar o 7Hinos sem internet",
    summary: "Baixe os hinos antes, em casa ou com internet.",
    steps: [
      "Com internet, monte o Culto de hoje e clique em Preparar para sem internet.",
      "Ou baixe um álbum inteiro pelo botão de download dele.",
      "Para levar tudo para outro computador: Ajustes → Levar para outro computador (pendrive).",
    ],
    keywords: ["offline", "baixar", "download", "pendrive", "sem rede", "sem wifi", "internet caiu"],
    category: "Hinos",
    route: { name: "settings" },
  },
  {
    id: "minha-musica",
    title: "Cadastrar um corinho ou música que não está no acervo",
    summary: "Escreva a letra, divida em slides e, se quiser, ponha o áudio.",
    steps: ["Abra Minhas músicas.", "Clique em Nova música.", "Escreva a letra separando as estrofes por uma linha em branco.", "Salve. Ela aparece na busca como qualquer hino."],
    keywords: ["corinho", "nova musica", "criar", "cadastrar", "adicionar letra", "musica nova", "editar letra", "corrigir"],
    category: "Hinos",
    route: { name: "mysongs" },
  },

  // Telas
  {
    id: "ligar-telao",
    title: "Ligar o telão, a TV ou o projetor",
    summary: "Conecte o cabo; o 7Hinos encontra a tela e projeta nela.",
    steps: [
      "Ligue o cabo (HDMI) no computador e na TV ou projetor.",
      "Abra Telas (no canto de baixo, à esquerda).",
      "Clique em Projetar aqui na tela certa.",
      "Se a TV não aparecer na lista, clique em Telão em janela, arraste a janela para a TV e dê dois cliques nela.",
    ],
    keywords: ["tv", "televisao", "projetor", "datashow", "data show", "hdmi", "segunda tela", "monitor", "telão", "nao aparece"],
    category: "Telas",
    route: { name: "displays" },
  },
  {
    id: "retorno",
    title: "Usar uma TV como retorno (para quem canta ou prega)",
    summary: "O retorno mostra a letra grande, a próxima estrofe e as horas.",
    steps: [
      "Abra Telas.",
      "Na tela que fica de frente para o púlpito, clique em Usar como retorno.",
      "Em Tela de retorno, ligue ou desligue a próxima letra, as horas e o fundo.",
    ],
    keywords: ["retorno", "monitor de palco", "pulpito", "pregador", "cantor", "horas", "relogio no pulpito"],
    category: "Telas",
    route: { name: "displays" },
  },
  {
    id: "reabrir-telas",
    title: "O telão travou ou ficou preto",
    summary: "Reabrir as telas resolve sem fechar o programa.",
    steps: ["No painel da direita, clique no botão de recarregar (a setinha em círculo) ao lado do nome da tela.", "Ou abra Telas e clique em Reabrir telas.", "O vídeo que estiver tocando continua."],
    keywords: ["travou", "congelou", "parou", "nao mostra", "tela preta", "bug", "erro", "reiniciar tela"],
    category: "Problemas",
    route: { name: "displays" },
  },
  {
    id: "perfis-telas",
    title: "Salvar a arrumação das telas (perfil)",
    summary: "Guarde telão + retorno e abra tudo com um clique.",
    steps: ["Abra as telas do jeito que gosta, em Telas.", "Dê um nome ao perfil e clique em Salvar as telas abertas.", "Marque a estrela para abrir sozinho quando o 7Hinos começar."],
    keywords: ["perfil", "salvar telas", "configuracao", "sempre abrir", "automatico"],
    category: "Telas",
    route: { name: "displays" },
  },
  {
    id: "celular",
    title: "Controlar pelo celular",
    summary: "Passe as estrofes e abra hinos pelo celular, no Wi‑Fi da igreja.",
    steps: [
      "O computador e o celular precisam estar na mesma rede (Wi‑Fi ou roteador).",
      "Abra Ajustes → Controle pelo celular e ligue.",
      "Leia o QR code com a câmera do celular.",
    ],
    keywords: ["celular", "telefone", "controle remoto", "wifi", "qr code", "smartphone", "remoto"],
    category: "Telas",
    route: { name: "settings" },
  },

  // Culto
  {
    id: "culto-de-hoje",
    title: "Montar o culto de hoje (a ordem do programa)",
    summary: "Coloque hinos, vídeos e momentos na ordem e siga um por um.",
    steps: [
      "Ao lado de cada hino ou vídeo, clique no ícone da lista para pôr no Culto de hoje.",
      "Abra Culto de hoje para mudar a ordem com as setas.",
      "Na hora, o painel da direita mostra sempre o próximo.",
    ],
    keywords: ["programa", "ordem", "lista", "setlist", "roteiro", "planejar", "programacao"],
    category: "Culto",
    route: { name: "setlist" },
  },
  {
    id: "liturgia",
    title: "Pôr momentos do culto (oração, sermão, avisos)",
    summary: "Use um modelo pronto ou acrescente momentos e anote quem dirige.",
    steps: [
      "Em Culto de hoje, clique em Modelo de programa (Culto divino, Escola Sabatina, Culto jovem).",
      "Ou clique em + Momento para acrescentar um só.",
      "Clique no lápis para anotar quem dirige ou mudar o tempo.",
      "Ao clicar em Começar num momento com tempo, o cronômetro do púlpito liga sozinho.",
    ],
    keywords: ["liturgia", "oracao", "sermao", "momento", "modelo", "culto divino", "escola sabatina", "anotacao", "quem prega"],
    category: "Culto",
    route: { name: "setlist" },
  },
  {
    id: "videos",
    title: "Mostrar um vídeo (Informativo, Provai e Vede)",
    summary: "Adicione o vídeo uma vez e mostre com um clique.",
    steps: [
      "Abra Vídeos e clique em Adicionar vídeos.",
      "Marque o tipo (Informativo Mundial das Missões, Provai e Vede) para ele virar o “de hoje”.",
      "Clique em Exibir. O som sai pelo telão.",
    ],
    keywords: ["video", "informativo", "missoes", "provai e vede", "filme", "mp4", "youtube", "clipe"],
    category: "Culto",
    route: { name: "videos" },
  },
  {
    id: "sorteio",
    title: "Fazer um sorteio no telão",
    summary: "Sorteie números ou nomes, com suspense e sem repetir.",
    steps: ["Abra Sorteio.", "Escolha Números (de… até…) ou Nomes (um por linha).", "Clique em Sortear no telão."],
    keywords: ["sortear", "brinde", "premio", "rifa", "aleatorio", "nome", "numero"],
    category: "Culto",
    route: { name: "draw" },
  },
  {
    id: "cronometro",
    title: "Usar o cronômetro do pregador",
    summary: "Mostra no retorno quanto tempo falta, em vermelho quando acaba.",
    steps: [
      "Abra Avisos e relógio → Para o púlpito.",
      "Escolha o tempo (ou Outro… para qualquer tempo) e clique em Começar.",
      "Para tirar, use o botão Tirar no painel da direita.",
    ],
    keywords: ["cronometro", "tempo", "timer", "pregador", "sermao", "relogio", "faltam", "minutos"],
    category: "Culto",
    route: { name: "screen" },
  },
  {
    id: "recado",
    title: "Mandar um recado para quem está no púlpito",
    summary: "Aparece só no retorno; a igreja não vê.",
    steps: ["Abra Avisos e relógio → Para o púlpito.", "Escreva o recado (ou toque num pronto, como “Faltam 5 minutos”) e clique em Enviar.", "Clique em Tirar quando não precisar mais."],
    keywords: ["recado", "mensagem", "avisar pregador", "bilhete", "faltam 5 minutos", "encerrar"],
    category: "Culto",
    route: { name: "screen" },
  },

  // Bíblia e apresentações
  {
    id: "biblia",
    title: "Mostrar um versículo da Bíblia",
    summary: "Digite a referência e projete versículo por versículo.",
    steps: ["Abra Bíblia.", "Digite a referência, por exemplo jo 3 16 ou Salmos 23.", "Clique no versículo para projetar e use as setas para passar."],
    keywords: ["biblia", "versiculo", "texto biblico", "leitura", "salmo", "capitulo", "passagem", "referencia"],
    category: "Bíblia e apresentações",
    route: { name: "bible" },
  },
  {
    id: "pdf",
    title: "Projetar o PDF ou os slides do pregador",
    summary: "Importe o PDF uma vez; depois funciona sem internet.",
    steps: [
      "Abra Apresentações e clique em PDF.",
      "Escolha o arquivo e espere preparar as páginas.",
      "Clique em Projetar e passe as páginas com as setas.",
      "Do PowerPoint: Arquivo → Salvar como → PDF, e importe o PDF.",
    ],
    keywords: ["pdf", "powerpoint", "ppt", "slides", "apresentacao", "pregador", "arquivo", "sermao"],
    category: "Bíblia e apresentações",
    route: { name: "decks" },
  },
  {
    id: "licao",
    title: "Projetar a lição da Escola Sabatina",
    summary: "Baixe a lição do trimestre e continue de onde parou.",
    steps: [
      "Abra Apresentações → Lição da Escola Sabatina.",
      "Clique em Ver lições do trimestre e depois em Baixar e abrir (ou Baixar todas).",
      "Na semana seguinte, use Continuar de onde parou.",
    ],
    keywords: ["licao", "escola sabatina", "trimestre", "estudo", "classe"],
    category: "Bíblia e apresentações",
    route: { name: "decks" },
  },
  {
    id: "zoom-caneta",
    title: "Aproximar, destacar ou riscar uma parte do slide",
    summary: "Zoom, foco, marca-texto, caneta e laser durante a apresentação.",
    steps: [
      "Com a apresentação aberta, use a barra embaixo da prévia.",
      "Zoom aproxima onde clicar; Ajustar à largura serve para PDF em pé.",
      "Foco escurece o resto; Marca-texto e Caneta desenham; Laser mostra um ponto vermelho.",
      "Esc solta a ferramenta; Limpar apaga tudo da página.",
    ],
    keywords: ["zoom", "aumentar", "ampliar", "caneta", "marca texto", "destacar", "laser", "riscar", "desenhar", "foco"],
    category: "Bíblia e apresentações",
    route: { name: "decks" },
  },

  // Avisos
  {
    id: "avisos",
    title: "Mostrar um aviso no telão",
    summary: "Salve os avisos da semana e projete com um clique.",
    steps: ["Abra Avisos e relógio.", "Em Avisos, clique em Novo aviso, escreva e salve.", "Clique em Projetar no aviso."],
    keywords: ["aviso", "anuncio", "recado da igreja", "informacao", "comunicado", "texto"],
    category: "Avisos",
    route: { name: "screen" },
  },
  {
    id: "contagem",
    title: "Contagem regressiva antes do culto",
    summary: "“O culto começa em 05:00” no telão.",
    steps: ["Abra Avisos e relógio.", "Em Contagem regressiva, escolha os minutos ou digite a hora de início.", "Clique em Começar."],
    keywords: ["contagem", "regressiva", "comeca em", "minutos", "antes do culto", "relogio"],
    category: "Avisos",
    route: { name: "screen" },
  },
  {
    id: "tela-espera",
    title: "Pôr o logo da igreja na tela de espera",
    summary: "O que o telão mostra quando nada está sendo projetado.",
    steps: ["Abra Avisos e relógio → Tela de espera.", "Escreva o nome da igreja e clique em Escolher imagem para o logo.", "Ligue Mostrar as horas no canto, se quiser."],
    keywords: ["logo", "tela de espera", "boas vindas", "nome da igreja", "imagem", "parado"],
    category: "Avisos",
    route: { name: "screen" },
  },
  {
    id: "aparencia",
    title: "Mudar as cores, a letra e o fundo do telão",
    summary: "Escolha a fonte, as cores e o fundo dos avisos e dos hinos.",
    steps: [
      "Para os hinos: Ajustes → Slides (letra, maiúsculas, tamanho).",
      "Para avisos, relógio e sorteio: Avisos e relógio → Aparência no telão.",
      "A bolinha colorida de arco-íris abre o seletor para qualquer cor.",
      "Não gostou? Voltar ao padrão, ao lado do título, deixa como o 7Hinos vem instalado.",
    ],
    keywords: ["cor", "fonte", "letra", "fundo", "degrade", "tamanho da letra", "maiuscula", "estilo", "aparencia", "restaurar", "padrao", "desfazer", "original"],
    category: "Avisos",
    route: { name: "screen" },
  },

  // Problemas
  {
    id: "sem-som",
    title: "O hino não tem som",
    summary: "Confira o volume, a internet e a saída de som.",
    steps: [
      "Veja se o volume do computador e da caixa de som estão ligados.",
      "Hinos que não foram baixados precisam de internet; baixe antes do culto.",
      "Se escolheu Só letra, não há som: escolha Cantado ou Playback.",
    ],
    keywords: ["sem som", "nao sai som", "mudo", "nao toca", "audio", "volume", "caixa de som", "silencio"],
    category: "Problemas",
  },
  {
    id: "diagnostico",
    title: "Pedir ajuda a quem entende (diagnóstico)",
    summary: "Copie o diagnóstico e envie junto com a dúvida.",
    steps: ["Abra Ajustes → Sobre.", "Clique em Copiar diagnóstico.", "Cole na mensagem para quem ajuda você (ou no GitHub do projeto)."],
    keywords: ["erro", "problema", "suporte", "ajuda", "bug", "nao funciona", "relatar"],
    category: "Problemas",
    route: { name: "settings" },
  },
];

export const articleById = (id: string) => ARTICLES.find((a) => a.id === id) ?? null;
